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
use std::path;

#[derive(Debug, Serialize, Clone)]
pub struct ControllerConfig {
    pub network_port: u16,

    pub consensus_port: u16,

    pub executor_port: u16,

    pub storage_port: u16,

    pub controller_port: u16,

    pub kms_port: u16,

    pub node_address: String,

    pub package_limit: u64,
}

impl ControllerConfig {
    fn write_to_file(&self, path: impl AsRef<path::Path>) {
        crate::util::write_to_file(&self, path, "controller".to_string());
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemConfigFile {
    pub version: u32,
    pub chain_id: String,
    // address of admin
    pub admin: String,
    pub block_interval: u32,
    pub validators: Vec<String>,
    pub block_limit: u64,
}

impl SystemConfigFile {
    fn write_to_file(&self, path: impl AsRef<path::Path>) {
        crate::util::write_to_file(&self, path, "system_config".to_string());
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GenesisBlock {
    pub timestamp: u64,
    pub prevhash: String,
}

impl GenesisBlock {
    fn write_to_file(&self, path: impl AsRef<path::Path>) {
        crate::util::write_to_file(&self, path, "genesis_block".to_string());
    }
}

#[cfg(test)]
mod controller_test {
    use super::*;
    use toml::Value;
    use crate::util::write_to_file;

    #[test]
    fn basic_test() {
        let _ = std::fs::remove_file("example/config.toml");

        let config = ControllerConfig {
            network_port: 51230,
            consensus_port: 51231,
            executor_port: 51232,
            storage_port: 51233,
            controller_port: 51234,
            kms_port: 51235,
            node_address: "0xe7b14f079c1db897568883f0323af5887c2feebb".to_string(),
            package_limit: 30000
        };

        config.write_to_file("example/config.toml");

        let genesis = GenesisBlock {
            timestamp: 1633765324292,
            prevhash: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string()
        };

        genesis.write_to_file("example/config.toml");
    }
}

