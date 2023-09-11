// Copyright 2023 The Turbo Cache Authors. All rights reserved.
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
use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::Mutex;
use tokio::select;
use tokio::sync::watch;
use tonic::{transport, Request, Streaming};

use action_messages::{ActionInfo, ActionInfoHashKey, ActionState, DEFAULT_EXECUTION_PRIORITY};
use common::log;
use error::{make_err, Code, Error, ResultExt};
use platform_property_manager::PlatformPropertyManager;
use proto::build::bazel::remote::execution::v2::{
    capabilities_client::CapabilitiesClient, digest_function, execution_client::ExecutionClient, ExecuteRequest,
    ExecutionPolicy, GetCapabilitiesRequest, WaitExecutionRequest,
};
use proto::google::longrunning::Operation;
use scheduler::ActionScheduler;

pub struct GrpcScheduler {
    capabilities_client: CapabilitiesClient<transport::Channel>,
    execution_client: ExecutionClient<transport::Channel>,
    platform_property_managers: Mutex<HashMap<String, Arc<PlatformPropertyManager>>>,
}

impl GrpcScheduler {
    pub fn new(config: &config::schedulers::GrpcScheduler) -> Result<Self, Error> {
        let endpoint = transport::Channel::balance_list(std::iter::once(
            transport::Endpoint::new(config.endpoint.clone())
                .err_tip(|| format!("Could not parse {} in GrpcScheduler", config.endpoint))?,
        ));

        Ok(Self {
            capabilities_client: CapabilitiesClient::new(endpoint.clone()),
            execution_client: ExecutionClient::new(endpoint),
            platform_property_managers: Mutex::new(HashMap::new()),
        })
    }

    async fn stream_state(mut result_stream: Streaming<Operation>) -> Result<watch::Receiver<Arc<ActionState>>, Error> {
        if let Some(initial_response) = result_stream
            .message()
            .await
            .err_tip(|| "Recieving response from upstream scheduler")?
        {
            let (tx, rx) = watch::channel(Arc::new(initial_response.try_into()?));
            tokio::spawn(async move {
                loop {
                    select!(
                        _ = tx.closed() => {
                            log::info!("Client disconnected in GrpcScheduler");
                            return;
                        }
                        Ok(Some(response)) = result_stream.message() => {
                            match response.try_into() {
                                Ok(response) => {
                                    if let Err(err) = tx.send(Arc::new(response)) {
                                        log::info!("Client disconnected in GrpcScheduler: {}", err);
                                        return;
                                    }
                                }
                                Err(err) => log::error!("Error converting response to ActionState in GrpcScheduler: {}", err),
                            }
                        }
                    )
                }
            });
            return Ok(rx);
        }
        Err(make_err!(Code::Internal, "Upstream scheduler didn't accept action."))
    }
}

#[async_trait]
impl ActionScheduler for GrpcScheduler {
    async fn get_platform_property_manager(&self, instance_name: &str) -> Result<Arc<PlatformPropertyManager>, Error> {
        if let Some(platform_property_manager) = self.platform_property_managers.lock().get(instance_name) {
            return Ok(platform_property_manager.clone());
        }

        // Not in the cache, lookup the capabilities with the upstream.
        let capabilities = self
            .capabilities_client
            .clone()
            .get_capabilities(GetCapabilitiesRequest {
                instance_name: instance_name.to_string(),
            })
            .await?
            .into_inner();
        let platform_property_manager = Arc::new(PlatformPropertyManager::new(
            capabilities
                .execution_capabilities
                .err_tip(|| "Unable to get execution properties in GrpcScheduler")?
                .supported_node_properties
                .iter()
                .map(|property| (property.clone(), config::schedulers::PropertyType::Exact))
                .collect(),
        ));

        self.platform_property_managers
            .lock()
            .insert(instance_name.to_string(), platform_property_manager.clone());
        Ok(platform_property_manager)
    }

    async fn add_action(&self, action_info: ActionInfo) -> Result<watch::Receiver<Arc<ActionState>>, Error> {
        let execution_policy = if action_info.priority == DEFAULT_EXECUTION_PRIORITY {
            None
        } else {
            Some(ExecutionPolicy {
                priority: action_info.priority,
            })
        };
        let request = ExecuteRequest {
            instance_name: action_info.instance_name().clone(),
            skip_cache_lookup: action_info.skip_cache_lookup,
            action_digest: Some(action_info.digest().into()),
            execution_policy,
            // TODO: Get me from the original request, not very important as we ignore it.
            results_cache_policy: None,
            digest_function: digest_function::Value::Sha256.into(),
        };
        let result_stream = self
            .execution_client
            .clone()
            .execute(Request::new(request))
            .await
            .err_tip(|| "Sending action to upstream scheduler")?
            .into_inner();
        Self::stream_state(result_stream).await
    }

    async fn find_existing_action(
        &self,
        unique_qualifier: &ActionInfoHashKey,
    ) -> Option<watch::Receiver<Arc<ActionState>>> {
        let request = WaitExecutionRequest {
            name: unique_qualifier.action_name(),
        };
        let result_stream = self
            .execution_client
            .clone()
            .wait_execution(Request::new(request))
            .await;
        if let Err(err) = result_stream {
            log::info!("Error response looking up action with upstream scheduler: {}", err);
            return None;
        }
        Self::stream_state(result_stream.unwrap().into_inner()).await.ok()
    }

    async fn clean_recently_completed_actions(&self) {}
}
