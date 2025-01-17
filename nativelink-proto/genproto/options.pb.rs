// Copyright 2022 The NativeLink Authors. All rights reserved.
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

// This file is @generated by prost-build.
/// Docs in java enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum OptionEffectTag {
    /// This option's effect or intent is unknown.
    Unknown = 0,
    /// This flag has literally no effect.
    NoOp = 1,
    LosesIncrementalState = 2,
    ChangesInputs = 3,
    AffectsOutputs = 4,
    BuildFileSemantics = 5,
    BazelInternalConfiguration = 6,
    LoadingAndAnalysis = 7,
    Execution = 8,
    HostMachineResourceOptimizations = 9,
    EagernessToExit = 10,
    BazelMonitoring = 11,
    TerminalOutput = 12,
    ActionCommandLines = 13,
    TestRunner = 14,
}
impl OptionEffectTag {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unknown => "UNKNOWN",
            Self::NoOp => "NO_OP",
            Self::LosesIncrementalState => "LOSES_INCREMENTAL_STATE",
            Self::ChangesInputs => "CHANGES_INPUTS",
            Self::AffectsOutputs => "AFFECTS_OUTPUTS",
            Self::BuildFileSemantics => "BUILD_FILE_SEMANTICS",
            Self::BazelInternalConfiguration => "BAZEL_INTERNAL_CONFIGURATION",
            Self::LoadingAndAnalysis => "LOADING_AND_ANALYSIS",
            Self::Execution => "EXECUTION",
            Self::HostMachineResourceOptimizations => {
                "HOST_MACHINE_RESOURCE_OPTIMIZATIONS"
            }
            Self::EagernessToExit => "EAGERNESS_TO_EXIT",
            Self::BazelMonitoring => "BAZEL_MONITORING",
            Self::TerminalOutput => "TERMINAL_OUTPUT",
            Self::ActionCommandLines => "ACTION_COMMAND_LINES",
            Self::TestRunner => "TEST_RUNNER",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "UNKNOWN" => Some(Self::Unknown),
            "NO_OP" => Some(Self::NoOp),
            "LOSES_INCREMENTAL_STATE" => Some(Self::LosesIncrementalState),
            "CHANGES_INPUTS" => Some(Self::ChangesInputs),
            "AFFECTS_OUTPUTS" => Some(Self::AffectsOutputs),
            "BUILD_FILE_SEMANTICS" => Some(Self::BuildFileSemantics),
            "BAZEL_INTERNAL_CONFIGURATION" => Some(Self::BazelInternalConfiguration),
            "LOADING_AND_ANALYSIS" => Some(Self::LoadingAndAnalysis),
            "EXECUTION" => Some(Self::Execution),
            "HOST_MACHINE_RESOURCE_OPTIMIZATIONS" => {
                Some(Self::HostMachineResourceOptimizations)
            }
            "EAGERNESS_TO_EXIT" => Some(Self::EagernessToExit),
            "BAZEL_MONITORING" => Some(Self::BazelMonitoring),
            "TERMINAL_OUTPUT" => Some(Self::TerminalOutput),
            "ACTION_COMMAND_LINES" => Some(Self::ActionCommandLines),
            "TEST_RUNNER" => Some(Self::TestRunner),
            _ => None,
        }
    }
}
/// Docs in java enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum OptionMetadataTag {
    Experimental = 0,
    IncompatibleChange = 1,
    Deprecated = 2,
    Hidden = 3,
    Internal = 4,
    Immutable = 7,
}
impl OptionMetadataTag {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Experimental => "EXPERIMENTAL",
            Self::IncompatibleChange => "INCOMPATIBLE_CHANGE",
            Self::Deprecated => "DEPRECATED",
            Self::Hidden => "HIDDEN",
            Self::Internal => "INTERNAL",
            Self::Immutable => "IMMUTABLE",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "EXPERIMENTAL" => Some(Self::Experimental),
            "INCOMPATIBLE_CHANGE" => Some(Self::IncompatibleChange),
            "DEPRECATED" => Some(Self::Deprecated),
            "HIDDEN" => Some(Self::Hidden),
            "INTERNAL" => Some(Self::Internal),
            "IMMUTABLE" => Some(Self::Immutable),
            _ => None,
        }
    }
}
