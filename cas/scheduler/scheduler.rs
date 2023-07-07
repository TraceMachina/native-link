// Copyright 2022 The Turbo Cache Authors. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::cmp;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use std::time::SystemTime;

use async_lock::Mutex;
use lru::LruCache;
use rand::{thread_rng, Rng};
use tokio::sync::watch;

use action_messages::{ActionInfo, ActionInfoHashKey, ActionResult, ActionStage, ActionState, ExecutionMetadata};
use common::{log, DigestInfo};
use config::schedulers::SimpleScheduler;
use error::{error_if, make_err, make_input_err, Code, Error, ResultExt};
use platform_property_manager::PlatformPropertyManager;
use worker::{Worker, WorkerId, WorkerTimestamp, WorkerUpdate};

/// Default timeout for workers in seconds.
/// If this changes, remember to change the documentation in the config.
const DEFAULT_WORKER_TIMEOUT_S: u64 = 5;

/// Default times a job can retry before failing.
/// If this changes, remember to change the documentation in the config.
const DEFAULT_MAX_JOB_RETRIES: usize = 3;

/// Exit code sent if there is an internal error.
pub const INTERNAL_ERROR_EXIT_CODE: i32 = -178;

/// An action that is being awaited on and last known state.
struct AwaitedAction {
    action_info: Arc<ActionInfo>,
    current_state: Arc<ActionState>,
    notify_channel: watch::Sender<Arc<ActionState>>,

    /// Number of attempts the job has been tried.
    attempts: usize,
    /// Possible last error set by the worker. If empty and attempts is set, it may be due to
    /// something like a worker timeout.
    last_error: Option<Error>,
}

/// Holds the relationship of a worker that is executing a specific action.
struct RunningAction {
    worker_id: WorkerId,
    action: AwaitedAction,
}

struct Workers {
    workers: LruCache<WorkerId, Worker>,
}

impl Workers {
    fn new() -> Self {
        Self {
            workers: LruCache::unbounded(),
        }
    }

    /// Refreshes the lifetime of the worker with the given timestamp.
    fn refresh_lifetime(&mut self, worker_id: &WorkerId, timestamp: WorkerTimestamp) -> Result<(), Error> {
        let worker = self
            .workers
            .get_mut(worker_id)
            .ok_or_else(|| make_input_err!("Worker not found in worker map in refresh_lifetime() {}", worker_id))?;
        error_if!(
            worker.last_update_timestamp > timestamp,
            "Worker already had a timestamp of {}, but tried to update it with {}",
            worker.last_update_timestamp,
            timestamp
        );
        worker.last_update_timestamp = timestamp;
        Ok(())
    }

    /// Adds a worker to the pool.
    /// Note: This function will not do any task matching.
    fn add_worker(&mut self, worker: Worker) -> Result<(), Error> {
        let worker_id = worker.id;
        self.workers.put(worker_id, worker);

        // Worker is not cloneable, and we do not want to send the initial connection results until
        // we have added it to the map, or we might get some strange race conditions due to the way
        // the multi-threaded runtime works.
        let worker = self.workers.peek_mut(&worker_id).unwrap();
        let res = worker
            .send_initial_connection_result()
            .err_tip(|| "Failed to send initial connection result to worker");
        if let Err(e) = &res {
            log::error!(
                "Worker connection appears to have been closed while adding to pool : {:?}",
                e
            );
        }
        res
    }

    /// Removes worker from pool.
    /// Note: The caller is responsible for any rescheduling of any tasks that might be
    /// running.
    fn remove_worker(&mut self, worker_id: &WorkerId) -> Option<Worker> {
        self.workers.pop(worker_id)
    }

    /// Attempts to find a worker that is capable of running this action.
    // TODO(blaise.bruer) This algorithm is not very efficient. Simple testing using a tree-like
    // structure showed worse performance on a 10_000 worker * 7 properties * 1000 queued tasks
    // simulation of worst cases in a single threaded environment.
    fn find_worker_for_action_mut<'a>(&'a mut self, awaited_action: &AwaitedAction) -> Option<&'a mut Worker> {
        assert!(matches!(awaited_action.current_state.stage, ActionStage::Queued));
        let action_properties = &awaited_action.action_info.platform_properties;
        return self.workers.iter_mut().find_map(|(_, w)| {
            if action_properties.is_satisfied_by(&w.platform_properties) {
                Some(w)
            } else {
                None
            }
        });
    }
}

/// Simple helper type to help with self-documentation.
type ShouldRunAgain = bool;

struct SchedulerImpl {
    // BTreeMap uses `cmp` to do it's comparisons, this is a problem because we want to sort our
    // actions based on priority and insert timestamp but also want to find and join new actions
    // onto already executing (or queued) actions. We don't know the insert timestamp of queued
    // actions, so we won't be able to find it in a BTreeMap without iterating the entire map. To
    // get around this issue, we use two containers, one that will search using `Eq` which will
    // only match on the `unique_qualifier` field, which ignores fields that would prevent
    // multiplexing, and another which uses `Ord` for sorting.
    //
    // Important: These two fields must be kept in-sync, so if you modify one, you likely need to
    // modify the other.
    queued_actions_set: HashSet<Arc<ActionInfo>>,
    queued_actions: BTreeMap<Arc<ActionInfo>, AwaitedAction>,
    workers: Workers,
    active_actions: HashMap<Arc<ActionInfo>, RunningAction>,
    /// Timeout of how long to evict workers if no response in this given amount of time in seconds.
    worker_timeout_s: u64,
    /// Default times a job can retry before failing.
    max_job_retries: usize,
}

impl SchedulerImpl {
    /// Attempts to find a worker to execute an action and begins executing it.
    /// If an action is already running that is cacheable it may merge this action
    /// with the results and state changes of the already running action.
    /// If the task cannot be executed immediately it will be queued for execution
    /// based on priority and other metrics.
    /// All further updates to the action will be provided through `listener`.
    fn add_action(&mut self, action_info: ActionInfo) -> Result<watch::Receiver<Arc<ActionState>>, Error> {
        // Check to see if the action is running, if it is and cacheable, merge the actions.
        if let Some(running_action) = self.active_actions.get_mut(&action_info) {
            let rx = running_action.action.notify_channel.subscribe();
            running_action
                .action
                .notify_channel
                .send(running_action.action.current_state.clone())
                .unwrap();
            return Ok(rx);
        }

        // Check to see if the action is queued, if it is and cacheable, merge the actions.
        if let Some(mut arc_action_info) = self.queued_actions_set.take(&action_info) {
            let (original_action_info, queued_action) = self
                .queued_actions
                .remove_entry(&arc_action_info)
                .err_tip(|| "Internal error queued_actions and queued_actions_set should match")?;

            let new_priority = cmp::max(original_action_info.priority, action_info.priority);
            drop(original_action_info); // This increases the chance Arc::make_mut won't copy.

            // In the event our task is higher priority than the one already scheduled, increase
            // the priority of the scheduled one.
            Arc::make_mut(&mut arc_action_info).priority = new_priority;

            let rx = queued_action.notify_channel.subscribe();
            queued_action
                .notify_channel
                .send(queued_action.current_state.clone())
                .unwrap();

            // Even if we fail to send our action to the client, we need to add this action back to the
            // queue because it was remove earlier.
            self.queued_actions.insert(arc_action_info.clone(), queued_action);
            self.queued_actions_set.insert(arc_action_info);
            return Ok(rx);
        }

        // Action needs to be added to queue or is not cacheable.
        let action_info = Arc::new(action_info);
        let action_digest = action_info.digest().clone();

        // TODO(allada) This name field needs to be indexable. The client might perform operations
        // based on the name later. It cannot be the same index used as the workers though, because
        // we multiplex the same job requests from clients to the same worker, but one client should
        // not shutdown a job if another client is still waiting on it.
        let current_state = Arc::new(ActionState {
            name: format!("{:X}", thread_rng().gen::<u128>()),
            stage: ActionStage::Queued,
            action_digest,
        });

        let (tx, rx) = watch::channel(current_state.clone());
        self.queued_actions_set.insert(action_info.clone());
        self.queued_actions.insert(
            action_info.clone(),
            AwaitedAction {
                action_info,
                current_state,
                notify_channel: tx,
                attempts: 0,
                last_error: None,
            },
        );

        self.do_try_match();
        Ok(rx)
    }

    fn retry_action(&mut self, action_info: &Arc<ActionInfo>, worker_id: &WorkerId) {
        match self.active_actions.remove(action_info) {
            Some(running_action) => {
                let mut awaited_action = running_action.action;
                let send_result = if awaited_action.attempts >= self.max_job_retries {
                    Arc::make_mut(&mut awaited_action.current_state).stage = ActionStage::Error((
                        awaited_action.last_error.unwrap_or_else(|| {
                            make_err!(
                                Code::Internal,
                                "Job cancelled because it attempted to execute too many times and failed"
                            )
                        }),
                        ActionResult {
                            output_files: Default::default(),
                            output_folders: Default::default(),
                            output_directory_symlinks: Default::default(),
                            output_file_symlinks: Default::default(),
                            exit_code: INTERNAL_ERROR_EXIT_CODE,
                            stdout_digest: DigestInfo::empty_digest(),
                            stderr_digest: DigestInfo::empty_digest(),
                            execution_metadata: ExecutionMetadata {
                                worker: format!("{}", worker_id),
                                queued_timestamp: SystemTime::UNIX_EPOCH,
                                worker_start_timestamp: SystemTime::UNIX_EPOCH,
                                worker_completed_timestamp: SystemTime::UNIX_EPOCH,
                                input_fetch_start_timestamp: SystemTime::UNIX_EPOCH,
                                input_fetch_completed_timestamp: SystemTime::UNIX_EPOCH,
                                execution_start_timestamp: SystemTime::UNIX_EPOCH,
                                execution_completed_timestamp: SystemTime::UNIX_EPOCH,
                                output_upload_start_timestamp: SystemTime::UNIX_EPOCH,
                                output_upload_completed_timestamp: SystemTime::UNIX_EPOCH,
                            },
                            server_logs: Default::default(),
                        },
                    ));
                    awaited_action.notify_channel.send(awaited_action.current_state.clone())
                    // Do not put the action back in the queue here, as this action attempted to run too many
                    // times.
                } else {
                    Arc::make_mut(&mut awaited_action.current_state).stage = ActionStage::Queued;
                    let send_result = awaited_action.notify_channel.send(awaited_action.current_state.clone());
                    self.queued_actions_set.insert(action_info.clone());
                    self.queued_actions.insert(action_info.clone(), awaited_action);
                    send_result
                };

                if send_result.is_err() {
                    // Don't remove this task, instead we keep them around for a bit just in case
                    // the client disconnected and will reconnect and ask for same job to be executed
                    // again.
                    log::warn!(
                        "Action {} has no more listeners during evict_worker()",
                        action_info.digest().str()
                    );
                }
            }
            None => {
                log::error!("Worker stated it was running an action, but it was not in the active_actions : Worker: {:?}, ActionInfo: {:?}", worker_id, action_info);
            }
        }
    }

    /// Evicts the worker from the pool and puts items back into the queue if anything was being executed on it.
    /// Note: This will not call .do_try_match().
    fn immediate_evict_worker(&mut self, worker_id: &WorkerId) {
        if let Some(mut worker) = self.workers.remove_worker(&worker_id) {
            // We don't care if we fail to send message to worker, this is only a best attempt.
            let _ = worker.notify_update(WorkerUpdate::Disconnect);
            // We create a temporary Vec to avoid doubt about a possible code
            // path touching the worker.running_action_infos elsewhere.
            for action_info in worker.running_action_infos.drain() {
                self.retry_action(&action_info, &worker_id);
            }
        }
    }

    /// Wrapper to keep running in the event we could not complete all scheduling in one iteration.
    fn do_try_match(&mut self) {
        // Run do_try_match until it doesn't need to run again.
        while self.inner_do_try_match() {}
    }

    // TODO(blaise.bruer) This is an O(n*m) (aka n^2) algorithm. In theory we can create a map
    // of capabilities of each worker and then try and match the actions to the worker using
    // the map lookup (ie. map reduce).
    fn inner_do_try_match(&mut self) -> ShouldRunAgain {
        // TODO(blaise.bruer) This is a bit difficult because of how rust's borrow checker gets in
        // the way. We need to conditionally remove items from the `queued_action`. Rust is working
        // to add `drain_filter`, which would in theory solve this problem, but because we need
        // to iterate the items in reverse it becomes more difficult (and it is currently an
        // unstable feature [see: https://github.com/rust-lang/rust/issues/70530]).
        let action_infos: Vec<Arc<ActionInfo>> = self.queued_actions.keys().rev().cloned().collect();
        for action_info in action_infos {
            let awaited_action = match self.queued_actions.get(action_info.as_ref()) {
                Some(awaited_action) => awaited_action,
                _ => {
                    log::error!(
                        "queued_actions out of sync with itself for action {}",
                        action_info.digest().str()
                    );
                    continue;
                }
            };
            let worker = if let Some(worker) = self.workers.find_worker_for_action_mut(&awaited_action) {
                worker
            } else {
                // No worker found, check the next action to see if there's a
                // matching one for that.
                continue;
            };
            let worker_id = worker.id.clone();

            // Try to notify our worker of the new action to run, if it fails remove the worker from the
            // pool and try to find another worker.
            let notify_worker_result = worker.notify_update(WorkerUpdate::RunAction(action_info.clone()));
            if notify_worker_result.is_err() {
                // Remove worker, as it is no longer receiving messages and let it try to find another worker.
                log::warn!("Worker command failed, removing worker {}", worker_id);
                self.immediate_evict_worker(&worker_id);
                // After evicting the worker, all running actions have been
                // placed back in the queue, so start again now.
                return true;
            }

            // At this point everything looks good, so remove it from the queue and add it to active actions.
            let (action_info, mut awaited_action) = self.queued_actions.remove_entry(action_info.as_ref()).unwrap();
            assert!(
                self.queued_actions_set.remove(&action_info),
                "queued_actions_set should always have same keys as queued_actions"
            );
            Arc::make_mut(&mut awaited_action.current_state).stage = ActionStage::Executing;
            let send_result = awaited_action.notify_channel.send(awaited_action.current_state.clone());
            if send_result.is_err() {
                // Don't remove this task, instead we keep them around for a bit just in case
                // the client disconnected and will reconnect and ask for same job to be executed
                // again.
                log::warn!(
                    "Action {} has no more listeners",
                    awaited_action.action_info.digest().str()
                );
            }
            awaited_action.attempts += 1;
            self.active_actions.insert(
                action_info.clone(),
                RunningAction {
                    worker_id,
                    action: awaited_action,
                },
            );
        }
        false
    }

    fn update_action_with_internal_error(
        &mut self,
        worker_id: &WorkerId,
        action_info_hash_key: &ActionInfoHashKey,
        err: Error,
    ) {
        let (action_info, mut running_action) = match self.active_actions.remove_entry(action_info_hash_key) {
            Some((action_info, running_action)) => (action_info, running_action),
            None => {
                log::error!(
                    "Could not find action info in active actions : {:?}",
                    action_info_hash_key
                );
                return;
            }
        };

        if running_action.worker_id != *worker_id {
            log::error!(
                "Got a result from a worker that should not be running the action, Removing worker. Expected worker {} got worker {}",
                    running_action.worker_id, worker_id
            );
        } else {
            // Don't set the error on an action that's running somewhere else.
            log::warn!("Internal error for worker {}: {}", worker_id, err);
            running_action.action.last_error = Some(err);
        }

        // Now put it back. retry_action() needs it to be there to send errors properly.
        self.active_actions.insert(action_info.clone(), running_action);

        // Clear this action from the current worker.
        if let Some(worker) = self.workers.workers.get_mut(worker_id) {
            worker.complete_action(&action_info);
        }

        // Re-queue the action or fail on max attempts.
        self.retry_action(&action_info, &worker_id);
    }

    async fn update_action(
        &mut self,
        worker_id: &WorkerId,
        action_info_hash_key: &ActionInfoHashKey,
        action_stage: ActionStage,
    ) -> Result<(), Error> {
        if !action_stage.has_action_result() {
            let msg = format!(
                "Worker '{}' set the action_stage of running action {:?} to {:?}. Removing worker.",
                worker_id, action_info_hash_key, action_stage
            );
            log::error!("{}", msg);
            self.immediate_evict_worker(worker_id);
            return Err(make_input_err!("{}", msg));
        }

        let (action_info, mut running_action) =
            self.active_actions.remove_entry(action_info_hash_key).err_tip(|| {
                format!(
                    "Could not find action info in active actions : {:?}",
                    action_info_hash_key
                )
            })?;

        if running_action.worker_id != *worker_id {
            let msg = format!(
                "Got a result from a worker that should not be running the action, {}",
                format_args!(
                    "Removing worker. Expected worker {} got worker {}",
                    running_action.worker_id, worker_id
                )
            );
            log::error!("{}", msg);
            // First put it back in our active_actions or we will drop the task.
            self.active_actions.insert(action_info.clone(), running_action);
            self.immediate_evict_worker(worker_id);
            return Err(make_input_err!("{}", msg));
        }

        let did_complete = match action_stage {
            ActionStage::Completed(_) => true,
            _ => false,
        };

        Arc::make_mut(&mut running_action.action.current_state).stage = action_stage;

        let send_result = running_action
            .action
            .notify_channel
            .send(running_action.action.current_state);
        if send_result.is_err() {
            log::warn!(
                "Action {} has no more listeners during update_action()",
                action_info.digest().str()
            );
        }

        if did_complete {
            let worker = self
                .workers
                .workers
                .get_mut(worker_id)
                .ok_or_else(|| make_input_err!("WorkerId '{}' does not exist in workers map", worker_id))?;
            worker.complete_action(&action_info);
            self.do_try_match();
        }

        // TODO(allada) We should probably hold a small queue of recent actions for debugging.
        // Right now it will drop the action which also disconnects listeners here.
        Ok(())
    }
}

/// Engine used to manage the queued/running tasks and relationship with
/// the worker nodes. All state on how the workers and actions are interacting
/// should be held in this struct.
pub struct Scheduler {
    inner: Mutex<SchedulerImpl>,
    platform_property_manager: PlatformPropertyManager,
}

impl Scheduler {
    pub fn new(scheduler_cfg: &SimpleScheduler) -> Self {
        let platform_property_manager = PlatformPropertyManager::new(
            scheduler_cfg
                .supported_platform_properties
                .clone()
                .unwrap_or(HashMap::new()),
        );

        let mut worker_timeout_s = scheduler_cfg.worker_timeout_s;
        if worker_timeout_s == 0 {
            worker_timeout_s = DEFAULT_WORKER_TIMEOUT_S;
        }

        let mut max_job_retries = scheduler_cfg.max_job_retries;
        if max_job_retries == 0 {
            max_job_retries = DEFAULT_MAX_JOB_RETRIES;
        }

        Self {
            inner: Mutex::new(SchedulerImpl {
                queued_actions_set: HashSet::new(),
                queued_actions: BTreeMap::new(),
                workers: Workers::new(),
                active_actions: HashMap::new(),
                worker_timeout_s,
                max_job_retries,
            }),
            platform_property_manager,
        }
    }

    pub fn get_platform_property_manager(&self) -> &PlatformPropertyManager {
        &self.platform_property_manager
    }

    /// Adds a worker to the scheduler and begin using it to execute actions (when able).
    pub async fn add_worker(&self, worker: Worker) -> Result<(), Error> {
        let worker_id = worker.id.clone();
        let mut inner = self.inner.lock().await;
        let res = inner
            .workers
            .add_worker(worker)
            .err_tip(|| "Error while adding worker, removing from pool");
        if res.is_err() {
            inner.immediate_evict_worker(&worker_id);
            return res;
        }
        inner.do_try_match();
        Ok(())
    }

    /// Adds an action to the scheduler for remote execution.
    pub async fn add_action(&self, action_info: ActionInfo) -> Result<watch::Receiver<Arc<ActionState>>, Error> {
        let mut inner = self.inner.lock().await;
        inner.add_action(action_info)
    }

    pub async fn update_action_with_internal_error(
        &self,
        worker_id: &WorkerId,
        action_info_hash_key: &ActionInfoHashKey,
        err: Error,
    ) {
        let mut inner = self.inner.lock().await;
        inner.update_action_with_internal_error(worker_id, action_info_hash_key, err)
    }

    /// Adds an action to the scheduler for remote execution.
    pub async fn update_action(
        &self,
        worker_id: &WorkerId,
        action_info_hash_key: &ActionInfoHashKey,
        action_stage: ActionStage,
    ) -> Result<(), Error> {
        let mut inner = self.inner.lock().await;
        inner.update_action(worker_id, action_info_hash_key, action_stage).await
    }

    /// Event for when the keep alive message was received from the worker.
    pub async fn worker_keep_alive_received(
        &self,
        worker_id: &WorkerId,
        timestamp: WorkerTimestamp,
    ) -> Result<(), Error> {
        let mut inner = self.inner.lock().await;
        inner
            .workers
            .refresh_lifetime(worker_id, timestamp)
            .err_tip(|| "Error refreshing lifetime in worker_keep_alive_received()")
    }

    /// Removes worker from pool and reschedule any tasks that might be running on it.
    pub async fn remove_worker(&self, worker_id: WorkerId) {
        let mut inner = self.inner.lock().await;
        inner.immediate_evict_worker(&worker_id);
        inner.do_try_match();
    }

    pub async fn remove_timedout_workers(&self, now_timestamp: WorkerTimestamp) -> Result<(), Error> {
        let mut inner = self.inner.lock().await;
        // Items should be sorted based on last_update_timestamp, so we don't need to iterate the entire
        // map most of the time.
        let worker_ids_to_remove: Vec<WorkerId> = inner
            .workers
            .workers
            .iter()
            .rev()
            .map_while(|(worker_id, worker)| {
                if worker.last_update_timestamp <= now_timestamp - inner.worker_timeout_s {
                    Some(*worker_id)
                } else {
                    None
                }
            })
            .collect();
        for worker_id in worker_ids_to_remove.as_slice() {
            log::warn!("Worker {} timed out, removing from pool", worker_id);
            inner.immediate_evict_worker(&worker_id);
        }
        if !worker_ids_to_remove.is_empty() {
            inner.do_try_match();
        }
        Ok(())
    }

    /// Checks to see if the worker exists in the worker pool. Should only be used in unit tests.
    pub async fn contains_worker_for_test(&self, worker_id: &WorkerId) -> bool {
        let inner = self.inner.lock().await;
        inner.workers.workers.contains(worker_id)
    }

    /// A unit test function used to send the keep alive message to the worker from the server.
    pub async fn send_keep_alive_to_worker_for_test(&self, worker_id: &WorkerId) -> Result<(), Error> {
        let mut inner = self.inner.lock().await;
        let worker = inner
            .workers
            .workers
            .get_mut(worker_id)
            .ok_or_else(|| make_input_err!("WorkerId '{}' does not exist in workers map", worker_id))?;
        worker.keep_alive()
    }
}
