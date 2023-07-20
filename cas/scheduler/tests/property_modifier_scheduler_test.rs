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
use std::time::UNIX_EPOCH;

use tokio::{join, sync::watch};

use action_messages::{ActionInfoHashKey, ActionStage, ActionState};
use common::DigestInfo;
use config::schedulers::{PlatformPropertyAddition, PropertyModification, PropertyType};
use error::Error;
use mock_scheduler::MockActionScheduler;
use platform_property_manager::{PlatformPropertyManager, PlatformPropertyValue};
use property_modifier_scheduler::PropertyModifierScheduler;
use scheduler::ActionScheduler;
use scheduler_utils::{make_base_action_info, INSTANCE_NAME};

struct TestContext {
    mock_scheduler: Arc<MockActionScheduler>,
    modifier_scheduler: PropertyModifierScheduler,
}

fn make_modifier_scheduler(modifications: Vec<PropertyModification>) -> TestContext {
    let mock_scheduler = Arc::new(MockActionScheduler::new());
    let config = config::schedulers::PropertyModifierScheduler {
        modifications,
        scheduler: Box::new(config::schedulers::SchedulerConfig::simple(
            config::schedulers::SimpleScheduler::default(),
        )),
    };
    let modifier_scheduler = PropertyModifierScheduler::new(&config, mock_scheduler.clone());
    TestContext {
        mock_scheduler,
        modifier_scheduler,
    }
}

#[cfg(test)]
mod property_modifier_scheduler_tests {
    use super::*;
    use pretty_assertions::assert_eq; // Must be declared in every module.

    #[tokio::test]
    async fn platform_property_manager_call_passed() -> Result<(), Error> {
        let context = make_modifier_scheduler(vec![]);
        let platform_property_manager = Arc::new(PlatformPropertyManager::new(HashMap::new()));
        let instance_name = INSTANCE_NAME.to_string();
        let (actual_manager, actual_instance_name) = join!(
            context.modifier_scheduler.get_platform_property_manager(&instance_name),
            context
                .mock_scheduler
                .expect_get_platform_property_manager(Ok(platform_property_manager.clone())),
        );
        assert_eq!(Arc::as_ptr(&platform_property_manager), Arc::as_ptr(&actual_manager?));
        assert_eq!(instance_name, actual_instance_name);
        Ok(())
    }

    #[tokio::test]
    async fn add_action_adds_property() -> Result<(), Error> {
        let name = "name".to_string();
        let value = "value".to_string();
        let context = make_modifier_scheduler(vec![PropertyModification::Add(PlatformPropertyAddition {
            name: name.clone(),
            value: value.clone(),
        })]);
        let action_info = make_base_action_info(UNIX_EPOCH);
        let (_forward_watch_channel_tx, forward_watch_channel_rx) = watch::channel(Arc::new(ActionState {
            unique_qualifier: action_info.unique_qualifier.clone(),
            stage: ActionStage::Queued,
        }));
        let platform_property_manager = Arc::new(PlatformPropertyManager::new(HashMap::from([(
            name.clone(),
            PropertyType::Exact,
        )])));
        let (_, _, action_info) = join!(
            context.modifier_scheduler.add_action(action_info),
            context
                .mock_scheduler
                .expect_get_platform_property_manager(Ok(platform_property_manager)),
            context.mock_scheduler.expect_add_action(Ok(forward_watch_channel_rx)),
        );
        assert_eq!(
            HashMap::from([(name, PlatformPropertyValue::Exact(value))]),
            action_info.platform_properties.properties
        );
        Ok(())
    }

    #[tokio::test]
    async fn add_action_overwrites_property() -> Result<(), Error> {
        let name = "name".to_string();
        let original_value = "value".to_string();
        let replaced_value = "replaced".to_string();
        let context = make_modifier_scheduler(vec![PropertyModification::Add(PlatformPropertyAddition {
            name: name.clone(),
            value: replaced_value.clone(),
        })]);
        let mut action_info = make_base_action_info(UNIX_EPOCH);
        action_info
            .platform_properties
            .properties
            .insert(name.clone(), PlatformPropertyValue::Unknown(original_value));
        let (_forward_watch_channel_tx, forward_watch_channel_rx) = watch::channel(Arc::new(ActionState {
            unique_qualifier: action_info.unique_qualifier.clone(),
            stage: ActionStage::Queued,
        }));
        let platform_property_manager = Arc::new(PlatformPropertyManager::new(HashMap::from([(
            name.clone(),
            PropertyType::Exact,
        )])));
        let (_, _, action_info) = join!(
            context.modifier_scheduler.add_action(action_info),
            context
                .mock_scheduler
                .expect_get_platform_property_manager(Ok(platform_property_manager)),
            context.mock_scheduler.expect_add_action(Ok(forward_watch_channel_rx)),
        );
        assert_eq!(
            HashMap::from([(name, PlatformPropertyValue::Exact(replaced_value))]),
            action_info.platform_properties.properties
        );
        Ok(())
    }

    #[tokio::test]
    async fn add_action_property_added_after_remove() -> Result<(), Error> {
        let name = "name".to_string();
        let value = "value".to_string();
        let context = make_modifier_scheduler(vec![
            PropertyModification::Remove(name.clone()),
            PropertyModification::Add(PlatformPropertyAddition {
                name: name.clone(),
                value: value.clone(),
            }),
        ]);
        let action_info = make_base_action_info(UNIX_EPOCH);
        let (_forward_watch_channel_tx, forward_watch_channel_rx) = watch::channel(Arc::new(ActionState {
            unique_qualifier: action_info.unique_qualifier.clone(),
            stage: ActionStage::Queued,
        }));
        let platform_property_manager = Arc::new(PlatformPropertyManager::new(HashMap::from([(
            name.clone(),
            PropertyType::Exact,
        )])));
        let (_, _, action_info) = join!(
            context.modifier_scheduler.add_action(action_info),
            context
                .mock_scheduler
                .expect_get_platform_property_manager(Ok(platform_property_manager)),
            context.mock_scheduler.expect_add_action(Ok(forward_watch_channel_rx)),
        );
        assert_eq!(
            HashMap::from([(name, PlatformPropertyValue::Exact(value))]),
            action_info.platform_properties.properties
        );
        Ok(())
    }

    #[tokio::test]
    async fn add_action_property_remove_after_add() -> Result<(), Error> {
        let name = "name".to_string();
        let value = "value".to_string();
        let context = make_modifier_scheduler(vec![
            PropertyModification::Add(PlatformPropertyAddition {
                name: name.clone(),
                value: value.clone(),
            }),
            PropertyModification::Remove(name.clone()),
        ]);
        let action_info = make_base_action_info(UNIX_EPOCH);
        let (_forward_watch_channel_tx, forward_watch_channel_rx) = watch::channel(Arc::new(ActionState {
            unique_qualifier: action_info.unique_qualifier.clone(),
            stage: ActionStage::Queued,
        }));
        let platform_property_manager = Arc::new(PlatformPropertyManager::new(HashMap::from([(
            name,
            PropertyType::Exact,
        )])));
        let (_, _, action_info) = join!(
            context.modifier_scheduler.add_action(action_info),
            context
                .mock_scheduler
                .expect_get_platform_property_manager(Ok(platform_property_manager)),
            context.mock_scheduler.expect_add_action(Ok(forward_watch_channel_rx)),
        );
        assert_eq!(HashMap::from([]), action_info.platform_properties.properties);
        Ok(())
    }

    #[tokio::test]
    async fn add_action_property_remove() -> Result<(), Error> {
        let name = "name".to_string();
        let value = "value".to_string();
        let context = make_modifier_scheduler(vec![PropertyModification::Remove(name.clone())]);
        let mut action_info = make_base_action_info(UNIX_EPOCH);
        action_info
            .platform_properties
            .properties
            .insert(name, PlatformPropertyValue::Unknown(value));
        let (_forward_watch_channel_tx, forward_watch_channel_rx) = watch::channel(Arc::new(ActionState {
            unique_qualifier: action_info.unique_qualifier.clone(),
            stage: ActionStage::Queued,
        }));
        let platform_property_manager = Arc::new(PlatformPropertyManager::new(HashMap::new()));
        let (_, _, action_info) = join!(
            context.modifier_scheduler.add_action(action_info),
            context
                .mock_scheduler
                .expect_get_platform_property_manager(Ok(platform_property_manager)),
            context.mock_scheduler.expect_add_action(Ok(forward_watch_channel_rx)),
        );
        assert_eq!(HashMap::from([]), action_info.platform_properties.properties);
        Ok(())
    }

    #[tokio::test]
    async fn find_existing_action_call_passed() -> Result<(), Error> {
        let context = make_modifier_scheduler(vec![]);
        let action_name = ActionInfoHashKey {
            instance_name: "instance".to_string(),
            digest: DigestInfo::new([8; 32], 1),
            salt: 1000,
        };
        let (actual_result, actual_action_name) = join!(
            context.modifier_scheduler.find_existing_action(&action_name),
            context.mock_scheduler.expect_find_existing_action(None),
        );
        assert_eq!(true, actual_result.is_none());
        assert_eq!(action_name, actual_action_name);
        Ok(())
    }
}
