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

use serde_derive::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ControllerConfig {
    pub controller_port: u16,

    pub network_port: u16,

    pub consensus_port: u16,

    pub storage_port: u16,

    pub kms_port: u16,

    pub executor_port: u16,

    pub node_address: String,

    pub package_limit: u64,
}

impl ControllerConfig {
    pub fn new(
        network_port: u16,
        consensus_port: u16,
        executor_port: u16,
        storage_port: u16,
        controller_port: u16,
        kms_port: u16,
        node_address: String,
        package_limit: u64,
    ) -> Self {
        Self {
            controller_port,
            network_port,
            consensus_port,
            storage_port,
            kms_port,
            executor_port,
            node_address,
            package_limit,
        }
    }
}

#[cfg(test)]
mod controller_test {
    use super::*;

    #[test]
    fn basic_test() {
        let config = ControllerConfig::new(
            51230,
            51231,
            51232,
            51233,
            51234,
            51235,
            "0xe7b14f079c1db897568883f0323af5887c2feebb".to_string(),
            30000
        );
    }
}

