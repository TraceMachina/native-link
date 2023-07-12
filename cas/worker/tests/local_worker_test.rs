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

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use prost::Message;
use rand::{thread_rng, Rng};
use tokio::io::AsyncWriteExt;
use tonic::Response;

use action_messages::{ActionInfo, ActionInfoHashKey, ActionResult, ActionStage, ExecutionMetadata};
use common::{encode_stream_proto, fs, DigestInfo};
use config::cas_server::{LocalWorkerConfig, WrokerProperty};
use error::{make_input_err, Error};
use fast_slow_store::FastSlowStore;
use filesystem_store::FilesystemStore;
use local_worker::new_local_worker;
use local_worker_test_utils::{setup_grpc_stream, setup_local_worker};
use memory_store::MemoryStore;
use mock_running_actions_manager::MockRunningAction;
use platform_property_manager::PlatformProperties;
use proto::build::bazel::remote::execution::v2::platform::Property;
use proto::com::github::allada::turbo_cache::remote_execution::{
    execute_result, update_for_worker::Update, ConnectionResult, ExecuteResult, StartExecute, SupportedProperties,
    UpdateForWorker,
};

/// Get temporary path from either `TEST_TMPDIR` or best effort temp directory if
/// not set.
fn make_temp_path(data: &str) -> String {
    format!(
        "{}/{}/{}",
        env::var("TEST_TMPDIR").unwrap_or(env::temp_dir().to_str().unwrap().to_string()),
        thread_rng().gen::<u64>(),
        data
    )
}

#[cfg(test)]
mod local_worker_tests {
    use super::*;
    use pretty_assertions::assert_eq; // Must be declared in every module.

    #[ctor::ctor]
    fn init() {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
            .format_timestamp_millis()
            .init();
    }

    #[tokio::test]
    async fn platform_properties_smoke_test() -> Result<(), Error> {
        let mut platform_properties = HashMap::new();
        platform_properties.insert(
            "foo".to_string(),
            WrokerProperty::values(vec!["bar1".to_string(), "bar2".to_string()]),
        );
        platform_properties.insert(
            "baz".to_string(),
            // Note: new lines will result in two entries for same key.
            WrokerProperty::query_cmd("echo -e 'hello\ngoodbye'".to_string()),
        );
        let mut test_context = setup_local_worker(platform_properties).await;
        let streaming_response = test_context.maybe_streaming_response.take().unwrap();

        // Now wait for our client to send `.connect_worker()` (which has our platform properties).
        let mut supported_properties = test_context.client.expect_connect_worker(Ok(streaming_response)).await;
        // It is undefined which order these will be returned in, so we sort it.
        supported_properties.properties.sort_by_key(Message::encode_to_vec);
        assert_eq!(
            supported_properties,
            SupportedProperties {
                properties: vec![
                    Property {
                        name: "baz".to_string(),
                        value: "hello".to_string(),
                    },
                    Property {
                        name: "baz".to_string(),
                        value: "goodbye".to_string(),
                    },
                    Property {
                        name: "foo".to_string(),
                        value: "bar1".to_string(),
                    },
                    Property {
                        name: "foo".to_string(),
                        value: "bar2".to_string(),
                    }
                ]
            }
        );

        Ok(())
    }

    #[tokio::test]
    async fn reconnect_on_server_disconnect_test() -> Result<(), Box<dyn std::error::Error>> {
        let mut test_context = setup_local_worker(HashMap::new()).await;
        let streaming_response = test_context.maybe_streaming_response.take().unwrap();

        {
            // Ensure our worker connects and properties were sent.
            let props = test_context.client.expect_connect_worker(Ok(streaming_response)).await;
            assert_eq!(props, SupportedProperties::default());
        }

        // Disconnect our grpc stream.
        test_context.maybe_tx_stream.take().unwrap().abort();

        {
            // Client should try to auto reconnect and check our properties again.
            let (_, streaming_response) = setup_grpc_stream();
            let props = test_context.client.expect_connect_worker(Ok(streaming_response)).await;
            assert_eq!(props, SupportedProperties::default());
        }

        Ok(())
    }

    #[tokio::test]
    async fn kill_all_called_on_disconnect() -> Result<(), Box<dyn std::error::Error>> {
        let mut test_context = setup_local_worker(HashMap::new()).await;
        let streaming_response = test_context.maybe_streaming_response.take().unwrap();

        {
            // Ensure our worker connects and properties were sent.
            let props = test_context.client.expect_connect_worker(Ok(streaming_response)).await;
            assert_eq!(props, SupportedProperties::default());
        }

        // Handle registration (kill_all not called unless registered).
        let mut tx_stream = test_context.maybe_tx_stream.take().unwrap();
        {
            tx_stream
                .send_data(encode_stream_proto(&UpdateForWorker {
                    update: Some(Update::ConnectionResult(ConnectionResult {
                        worker_id: "foobar".to_string(),
                    })),
                })?)
                .await
                .map_err(|e| make_input_err!("Could not send : {:?}", e))?;
        }

        // Disconnect our grpc stream.
        tx_stream.abort();

        // Check that kill_all is called.
        test_context.actions_manager.expect_kill_all().await;

        Ok(())
    }

    #[tokio::test]
    async fn simple_worker_start_action_test() -> Result<(), Box<dyn std::error::Error>> {
        const SALT: u64 = 1000;

        let mut test_context = setup_local_worker(HashMap::new()).await;
        let streaming_response = test_context.maybe_streaming_response.take().unwrap();

        {
            // Ensure our worker connects and properties were sent.
            let props = test_context.client.expect_connect_worker(Ok(streaming_response)).await;
            assert_eq!(props, SupportedProperties::default());
        }

        let expected_worker_id = "foobar".to_string();

        let mut tx_stream = test_context.maybe_tx_stream.take().unwrap();
        {
            // First initialize our worker by sending the response to the connection request.
            tx_stream
                .send_data(encode_stream_proto(&UpdateForWorker {
                    update: Some(Update::ConnectionResult(ConnectionResult {
                        worker_id: expected_worker_id.clone(),
                    })),
                })?)
                .await
                .map_err(|e| make_input_err!("Could not send : {:?}", e))?;
        }

        let action_digest = DigestInfo::new([3u8; 32], 10);
        let action_info = ActionInfo {
            instance_name: "foo".to_string(),
            command_digest: DigestInfo::new([1u8; 32], 10),
            input_root_digest: DigestInfo::new([2u8; 32], 10),
            timeout: Duration::from_secs(1),
            platform_properties: PlatformProperties::default(),
            priority: 0,
            load_timestamp: SystemTime::UNIX_EPOCH,
            insert_timestamp: SystemTime::UNIX_EPOCH,
            unique_qualifier: ActionInfoHashKey {
                digest: action_digest,
                salt: SALT,
            },
            skip_cache_lookup: true,
        };

        {
            // Send execution request.
            tx_stream
                .send_data(encode_stream_proto(&UpdateForWorker {
                    update: Some(Update::StartAction(StartExecute {
                        execute_request: Some(action_info.into()),
                        salt: SALT,
                        queued_timestamp: None,
                    })),
                })?)
                .await
                .map_err(|e| make_input_err!("Could not send : {:?}", e))?;
        }
        let action_result = ActionResult {
            output_files: vec![],
            output_folders: vec![],
            output_file_symlinks: vec![],
            output_directory_symlinks: vec![],
            exit_code: 5,
            stdout_digest: DigestInfo::new([21u8; 32], 10),
            stderr_digest: DigestInfo::new([22u8; 32], 10),
            execution_metadata: ExecutionMetadata {
                worker: expected_worker_id.clone(),
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
            server_logs: HashMap::new(),
        };
        let running_action = Arc::new(MockRunningAction::new());

        // Send and wait for response from create_and_add_action to RunningActionsManager.
        test_context
            .actions_manager
            .expect_create_and_add_action(Ok(running_action.clone()))
            .await;

        // Now the RunningAction needs to send a series of state updates. This shortcuts them
        // into a single call (shortcut for prepare, execute, upload, collect_results, cleanup).
        running_action
            .simple_expect_get_finished_result(Ok(action_result.clone()))
            .await?;

        // Expect the action to be updated in the action cache.
        let (stored_digest, stored_result) = test_context.actions_manager.expect_cache_action_result().await;
        assert_eq!(stored_digest, action_digest);
        assert_eq!(stored_result, action_result.clone());

        // Now our client should be notified that our runner finished.
        let execution_response = test_context
            .client
            .expect_execution_response(Ok(Response::new(())))
            .await;

        // Now ensure the final results match our expectations.
        assert_eq!(
            execution_response,
            ExecuteResult {
                worker_id: expected_worker_id,
                action_digest: Some(action_digest.into()),
                salt: SALT,
                result: Some(execute_result::Result::ExecuteResponse(
                    ActionStage::Completed(action_result).into()
                )),
            }
        );

        Ok(())
    }

    #[tokio::test]
    async fn new_local_worker_creates_work_directory_test() -> Result<(), Box<dyn std::error::Error>> {
        let cas_store = Arc::new(FastSlowStore::new(
            &config::stores::FastSlowStore {
                // Note: These are not needed for this test, so we put dummy memory stores here.
                fast: config::stores::StoreConfig::memory(config::stores::MemoryStore::default()),
                slow: config::stores::StoreConfig::memory(config::stores::MemoryStore::default()),
            },
            Arc::new(
                <FilesystemStore>::new(&config::stores::FilesystemStore {
                    content_path: make_temp_path("content_path"),
                    temp_path: make_temp_path("temp_path"),
                    ..Default::default()
                })
                .await?,
            ),
            Arc::new(MemoryStore::new(&config::stores::MemoryStore::default())),
        ));
        let ac_store = Arc::new(MemoryStore::new(&config::stores::MemoryStore::default()));
        let work_directory = make_temp_path("foo");
        new_local_worker(
            Arc::new(LocalWorkerConfig {
                work_directory: work_directory.clone(),
                ..Default::default()
            }),
            cas_store,
            ac_store,
        )
        .await?;

        assert!(
            fs::metadata(work_directory).await.is_ok(),
            "Expected work_directory to be created"
        );

        Ok(())
    }

    #[tokio::test]
    async fn new_local_worker_removes_work_directory_before_start_test() -> Result<(), Box<dyn std::error::Error>> {
        let cas_store = Arc::new(FastSlowStore::new(
            &config::stores::FastSlowStore {
                // Note: These are not needed for this test, so we put dummy memory stores here.
                fast: config::stores::StoreConfig::memory(config::stores::MemoryStore::default()),
                slow: config::stores::StoreConfig::memory(config::stores::MemoryStore::default()),
            },
            Arc::new(
                <FilesystemStore>::new(&config::stores::FilesystemStore {
                    content_path: make_temp_path("content_path"),
                    temp_path: make_temp_path("temp_path"),
                    ..Default::default()
                })
                .await?,
            ),
            Arc::new(MemoryStore::new(&config::stores::MemoryStore::default())),
        ));
        let ac_store = Arc::new(MemoryStore::new(&config::stores::MemoryStore::default()));
        let work_directory = make_temp_path("foo");
        fs::create_dir_all(format!("{}/{}", work_directory, "another_dir")).await?;
        let mut file = fs::create_file(format!("{}/{}", work_directory, "foo.txt")).await?;
        file.write_all(b"Hello, world!").await?;
        new_local_worker(
            Arc::new(LocalWorkerConfig {
                work_directory: work_directory.clone(),
                ..Default::default()
            }),
            cas_store,
            ac_store,
        )
        .await?;

        let work_directory_path_buf = PathBuf::from(work_directory);

        assert!(
            work_directory_path_buf.read_dir()?.next().is_none(),
            "Expected work_directory to have removed all files and to be empty"
        );

        Ok(())
    }
}
