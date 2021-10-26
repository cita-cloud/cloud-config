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

use crate::constant::{CONTROLLER, DEFAULT_BLOCK_INTERVAL, DEFAULT_BLOCK_LIMIT, GENESIS_BLOCK, GRPC_PORT_BEGIN, PACKAGE_LIMIT, PRE_HASH, SYSTEM_CONFIG};
use serde::{Deserialize, Serialize};

use std::path;
use crate::traits::TomlWriter;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ControllerConfig {
    pub network_port: u16,

    pub consensus_port: u16,

    pub executor_port: u16,

    pub storage_port: u16,

    pub controller_port: u16,

    pub kms_port: u16,

    pub key_id: u64,

    pub node_address: String,

    pub package_limit: u64,
}

impl ControllerConfig {
    pub fn new(network_port: u16,
               key_id: u64,
               address: &str,
               package_limit: u64) -> Self {
        Self {
            network_port,
            consensus_port: network_port + 1,
            executor_port: network_port + 2,
            storage_port: network_port + 3,
            controller_port: network_port + 4,
            kms_port: network_port + 5,
            key_id,
            node_address: address.into(),
            package_limit,
        }
    }
}

impl TomlWriter for ControllerConfig {
    fn section(&self) -> String {
        CONTROLLER.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn default(version: u32, chain_id: String, admin: String, validators: Vec<String>) -> Self {
        Self {
            version,
            chain_id,
            admin,
            block_interval: DEFAULT_BLOCK_INTERVAL,
            validators,
            block_limit: DEFAULT_BLOCK_LIMIT,
        }
    }
}

impl TomlWriter for SystemConfigFile {
    fn section(&self) -> String {
        SYSTEM_CONFIG.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisBlock {
    pub timestamp: u64,
    pub prevhash: String,
}

fn timestamp() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .unwrap();
    let ms = since_the_epoch.as_secs() as i64 * 1000i64 + (since_the_epoch.subsec_nanos() as i64 / 1_000_000) as i64;
    ms as u64
}

impl GenesisBlock {
    pub fn default() -> Self {
        Self {
            timestamp: timestamp(),
            prevhash: PRE_HASH.to_string(),
        }
    }
}

impl TomlWriter for GenesisBlock {
    fn section(&self) -> String {
        GENESIS_BLOCK.to_string()
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
            key_id: 1,
            node_address: "0xe7b14f079c1db897568883f0323af5887c2feebb".into(),
            package_limit: 30000,
        };

        config.write("example");

        let genesis = GenesisBlock {
            timestamp: 1633765324292,
            prevhash: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        };

        genesis.write("example");
    }
}

