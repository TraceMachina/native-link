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

use config::schedulers::PropertyType;
use error::{make_input_err, Code, Error, ResultExt};
use proto::build::bazel::remote::execution::v2::Platform as ProtoPlatform;

/// `PlatformProperties` helps manage the configuration of platform properties to
/// keys and types. The scheduler uses these properties to decide what jobs
/// can be assigned to different workers. For example, if a job states it needs
/// a specific key, it will never be run on a worker that does not have at least
/// all the platform property keys configured on the worker.
///
/// Additional rules may be applied based on `PlatfromPropertyValue`.
#[derive(Eq, PartialEq, Clone, Debug, Default)]
pub struct PlatformProperties {
    pub properties: HashMap<String, PlatformPropertyValue>,
}

impl PlatformProperties {
    #[must_use]
    pub const fn new(map: HashMap<String, PlatformPropertyValue>) -> Self {
        Self { properties: map }
    }

    /// Determines if the worker's `PlatformProperties` is satisfied by this struct.
    #[must_use]
    pub fn is_satisfied_by(&self, worker_properties: &Self) -> bool {
        for (property, check_value) in &self.properties {
            if let Some(worker_value) = worker_properties.properties.get(property) {
                if !check_value.is_satisfied_by(worker_value) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

impl From<ProtoPlatform> for PlatformProperties {
    fn from(platform: ProtoPlatform) -> Self {
        let mut properties = HashMap::with_capacity(platform.properties.len());
        for property in platform.properties {
            properties.insert(property.name, PlatformPropertyValue::Unknown(property.value));
        }
        Self { properties }
    }
}

/// Holds the associated value of the key and type.
///
/// Exact    - Means the worker must have this exact value.
/// Minimum  - Means that workers must have at least this number available. When
///            a worker executes a task that has this value, the worker will have
///            this value subtracted from the available resources of the worker.
/// Priority - Means the worker is given this information, but does not restrict
///            what workers can take this value. However, the worker must have the
///            associated key present to be matched.
///            TODO(allada) In the future this will be used by the scheduler and
///            worker to cause the scheduler to prefer certain workers over others,
///            but not restrict them based on these values.
#[derive(Eq, PartialEq, Hash, Clone, Ord, PartialOrd, Debug)]
pub enum PlatformPropertyValue {
    Exact(String),
    Minimum(u64),
    Priority(String),
    Unknown(String),
}

impl PlatformPropertyValue {
    /// Same as `PlatformProperties::is_satisfied_by`, but on an individual value.
    #[must_use]
    pub fn is_satisfied_by(&self, worker_value: &Self) -> bool {
        if self == worker_value {
            return true;
        }
        match self {
            Self::Minimum(v) => {
                if let Self::Minimum(worker_v) = worker_value {
                    return worker_v >= v;
                }
                false
            }
            // Priority is used to pass info to the worker and not restrict which
            // workers can be selected, but might be used to prefer certain workers
            // over others.
            Self::Priority(_) => true,
            // Success exact case is handled above.
            Self::Exact(_) | Self::Unknown(_) => false,
        }
    }
}

/// Helps manage known properties and conversion into `PlatformPropertyValue`.
pub struct PlatformPropertyManager {
    known_properties: HashMap<String, PropertyType>,
}

impl PlatformPropertyManager {
    #[must_use]
    pub const fn new(known_properties: HashMap<String, PropertyType>) -> Self {
        Self { known_properties }
    }

    /// Returns the `known_properties` map.
    #[must_use]
    pub const fn get_known_properties(&self) -> &HashMap<String, PropertyType> {
        &self.known_properties
    }

    /// Given a specific key and value, returns the translated `PlatformPropertyValue`. This will
    /// automatically convert any strings to the type-value pairs of `PlatformPropertyValue` based
    /// on the configuration passed into the `PlatformPropertyManager` constructor.
    pub fn make_prop_value(&self, key: &str, value: &str) -> Result<PlatformPropertyValue, Error> {
        if let Some(prop_type) = self.known_properties.get(key) {
            return match prop_type {
                PropertyType::Minimum => Ok(PlatformPropertyValue::Minimum(value.parse::<u64>().err_tip_with_code(
                    |e| {
                        (
                            Code::InvalidArgument,
                            format!("Cannot convert to platform property to u64: {value} - {e}"),
                        )
                    },
                )?)),
                PropertyType::Exact => Ok(PlatformPropertyValue::Exact(value.to_string())),
                PropertyType::Priority => Ok(PlatformPropertyValue::Priority(value.to_string())),
            };
        }
        Err(make_input_err!("Unknown platform property '{}'", key))
    }
}
