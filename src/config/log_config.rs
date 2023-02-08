// Copyright Rivtower Technologies LLC.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub max_level: String,
    pub filter: String,
    pub service_name: String,
    pub rolling_file_path: Option<String>,
    pub agent_endpoint: Option<String>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            max_level: "info".to_owned(),
            filter: "info".to_owned(),
            service_name: Default::default(),
            rolling_file_path: Default::default(),
            agent_endpoint: Default::default(),
        }
    }
}
