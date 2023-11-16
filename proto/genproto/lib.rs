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

// *** DO NOT MODIFY ***
// This file is auto-generated. To update it, run:
// `bazel run proto:update_protos`


pub mod build {
  pub mod bazel {
    pub mod remote {
      pub mod execution {
        pub mod v2 {
          include!("build.bazel.remote.execution.v2.pb.rs");
        }
      }
    }
    pub mod semver {
      include!("build.bazel.semver.pb.rs");
    }
  }
}
pub mod com {
  pub mod github {
    pub mod trace_machina {
      pub mod turbo_cache {
        pub mod remote_execution {
          include!("com.github.trace_machina.turbo_cache.remote_execution.pb.rs");
        }
      }
    }
  }
}
pub mod google {
  pub mod api {
    include!("google.api.pb.rs");
  }
  pub mod bytestream {
    include!("google.bytestream.pb.rs");
  }
  pub mod longrunning {
    include!("google.longrunning.pb.rs");
  }
  pub mod rpc {
    include!("google.rpc.pb.rs");
  }
}

