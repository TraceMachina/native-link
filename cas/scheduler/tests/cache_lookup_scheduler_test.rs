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
use std::pin::Pin;
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use prost::Message;
use tokio::{self, join, sync::watch};
use tokio_stream::wrappers::WatchStream;
use tokio_stream::StreamExt;

use action_messages::{ActionResult, ActionStage, ActionState, DirectoryInfo};
use cache_lookup_scheduler::CacheLookupScheduler;
use common::DigestInfo;
use error::{Error, ResultExt};
use memory_store::MemoryStore;
use mock_scheduler::MockActionScheduler;
use platform_property_manager::PlatformPropertyManager;
use proto::build::bazel::remote::execution::v2::ActionResult as ProtoActionResult;
use scheduler::ActionScheduler;
use scheduler_utils::{make_base_action_info, INSTANCE_NAME};
use store::Store;

struct TestContext {
    mock_scheduler: Arc<MockActionScheduler>,
    ac_store: Arc<dyn Store>,
    cache_scheduler: CacheLookupScheduler,
}

fn make_cache_scheduler() -> Result<TestContext, Error> {
    let mock_scheduler = Arc::new(MockActionScheduler::new());
    let cas_store = Arc::new(MemoryStore::new(&config::stores::MemoryStore::default()));
    let ac_store = Arc::new(MemoryStore::new(&config::stores::MemoryStore::default()));
    let cache_scheduler = CacheLookupScheduler::new(cas_store, ac_store.clone(), mock_scheduler.clone())?;
    Ok(TestContext {
        mock_scheduler,
        ac_store,
        cache_scheduler,
    })
}

#[cfg(test)]
mod cache_lookup_scheduler_tests {
    use super::*;
    use pretty_assertions::assert_eq; // Must be declared in every module.

    #[tokio::test]
    async fn platform_property_manager_call_passed() -> Result<(), Error> {
        let context = make_cache_scheduler()?;
        let platform_property_manager = Arc::new(PlatformPropertyManager::new(HashMap::new()));
        let instance_name = INSTANCE_NAME.to_string();
        let (actual_manager, actual_instance_name) = join!(
            context.cache_scheduler.get_platform_property_manager(&instance_name),
            context
                .mock_scheduler
                .expect_get_platform_property_manager(Ok(platform_property_manager.clone())),
        );
        assert_eq!(Arc::as_ptr(&platform_property_manager), Arc::as_ptr(&actual_manager?));
        assert_eq!(instance_name, actual_instance_name);
        Ok(())
    }

    #[tokio::test]
    async fn add_action_does_cache_lookup() -> Result<(), Error> {
        let context = make_cache_scheduler()?;
        let action_info = make_base_action_info(UNIX_EPOCH);
        let action_result = ProtoActionResult::from(ActionResult::default());
        let store_pin = Pin::new(context.ac_store.as_ref());
        store_pin
            .update_oneshot(
                action_info.unique_qualifier.digest,
                action_result.encode_to_vec().into(),
            )
            .await?;
        let watch_channel = context
            .cache_scheduler
            .add_action("name".to_string(), action_info.clone())
            .await?;
        let mut watch_stream = WatchStream::new(watch_channel);
        if watch_stream.next().await.err_tip(|| "Getting initial state")?.stage != ActionStage::CacheCheck {
            panic!("Not performing a cache check");
        }
        let cached_action_state = watch_stream.next().await.err_tip(|| "Getting post-cache result")?;
        let ActionStage::CompletedFromCache(proto_action_result) = cached_action_state.stage.clone() else {
            panic!("Did not complete from cache");
        };
        assert_eq!(action_info.unique_qualifier.digest, cached_action_state.action_digest);
        assert_eq!(action_result, proto_action_result);
        Ok(())
    }

    #[tokio::test]
    async fn add_action_validates_outputs() -> Result<(), Error> {
        let context = make_cache_scheduler()?;
        let action_info = make_base_action_info(UNIX_EPOCH);
        let mut action_result = ActionResult::default();
        action_result.output_folders.push(DirectoryInfo {
            path: "".to_string(),
            tree_digest: DigestInfo {
                size_bytes: 1,
                packed_hash: [8; 32],
            },
        });
        let action_result = ProtoActionResult::from(action_result);
        let store_pin = Pin::new(context.ac_store.as_ref());
        store_pin
            .update_oneshot(
                action_info.unique_qualifier.digest,
                action_result.encode_to_vec().into(),
            )
            .await?;
        let (_forward_watch_channel_tx, forward_watch_channel_rx) = watch::channel(Arc::new(ActionState {
            name: "".to_string(),
            action_digest: action_info.unique_qualifier.digest,
            stage: ActionStage::Queued,
        }));
        let _ = join!(
            context.cache_scheduler.add_action("name".to_string(), action_info),
            context.mock_scheduler.expect_add_action(Ok(forward_watch_channel_rx))
        );
        Ok(())
    }

    #[tokio::test]
    async fn add_action_handles_skip_cache() -> Result<(), Error> {
        let context = make_cache_scheduler()?;
        let action_info = make_base_action_info(UNIX_EPOCH);
        let action_result = ProtoActionResult::from(ActionResult::default());
        let store_pin = Pin::new(context.ac_store.as_ref());
        store_pin
            .update_oneshot(
                action_info.unique_qualifier.digest,
                action_result.encode_to_vec().into(),
            )
            .await?;
        let (_forward_watch_channel_tx, forward_watch_channel_rx) = watch::channel(Arc::new(ActionState {
            name: "".to_string(),
            action_digest: action_info.unique_qualifier.digest,
            stage: ActionStage::Queued,
        }));
        let mut skip_cache_action = action_info.clone();
        skip_cache_action.skip_cache_lookup = true;
        let _ = join!(
            context
                .cache_scheduler
                .add_action("name".to_string(), skip_cache_action),
            context.mock_scheduler.expect_add_action(Ok(forward_watch_channel_rx))
        );
        Ok(())
    }

    #[tokio::test]
    async fn find_existing_action_call_passed() -> Result<(), Error> {
        let context = make_cache_scheduler()?;
        let action_name = "action";
        let (actual_result, actual_action_name) = join!(
            context.cache_scheduler.find_existing_action(action_name),
            context.mock_scheduler.expect_find_existing_action(None),
        );
        assert_eq!(true, actual_result.is_none());
        assert_eq!(action_name, actual_action_name);
        Ok(())
    }
}
