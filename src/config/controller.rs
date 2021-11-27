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

use crate::constant::{
    CONTROLLER, DEFAULT_BLOCK_INTERVAL, DEFAULT_BLOCK_LIMIT, GENESIS_BLOCK, PRE_HASH, SYSTEM_CONFIG,
};
use crate::traits::{TomlWriter, YmlWriter};
use serde::{Deserialize, Serialize};
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
    pub fn new(network_port: u16, key_id: u64, address: &str, package_limit: u64) -> Self {
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

impl YmlWriter for ControllerConfig {
    fn service(&self) -> String {
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



    pub fn set_admin(&mut self, admin: String) {
        self.admin = admin;
    }

    pub fn set_validators(&mut self, validators: Vec<String>) {
        self.validators = validators;
    }
}

impl TomlWriter for SystemConfigFile {
    fn section(&self) -> String {
        SYSTEM_CONFIG.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfigBuilder {
    pub version: u32,
    pub chain_id: String,
    // address of admin
    pub admin: String,
    pub block_interval: u32,
    pub validators: Vec<String>,
    pub block_limit: u64,
}

impl SystemConfigBuilder {

    pub fn new() -> Self {
        Self {
            version: 0,
            chain_id: "".to_string(),
            admin: "".to_string(),
            block_interval: DEFAULT_BLOCK_INTERVAL,
            validators: Vec::new(),
            block_limit: DEFAULT_BLOCK_LIMIT,
        }
    }

    pub fn version(&mut self, version: u32) -> &mut SystemConfigBuilder {
        self.version = version;
        self
    }

    pub fn chain_id(&mut self, chain_id: String) -> &mut SystemConfigBuilder {
        self.chain_id = chain_id;
        self
    }

    pub fn admin(&mut self, chain_id: String) -> &mut SystemConfigBuilder {
        self.chain_id = chain_id;
        self
    }

    pub fn block_interval(&mut self, block_interval: u32) -> &mut SystemConfigBuilder {
        self.block_interval = block_interval;
        self
    }

    pub fn validators(&mut self, validators: Vec<String>) -> &mut SystemConfigBuilder {
        self.validators = validators;
        self
    }

    pub fn block_limit(&mut self, block_limit: u64) -> &mut SystemConfigBuilder {
        self.block_limit = block_limit;
        self
    }

    pub fn build(&self) -> SystemConfigFile {
        SystemConfigFile {
            version: self.version,
            chain_id: self.chain_id.clone(),
            admin: self.admin.clone(),
            block_interval: self.block_interval,
            validators: self.validators.clone(),
            block_limit: self.block_limit,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisBlock {
    pub timestamp: u64,
    pub prevhash: String,
}

fn timestamp() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    let ms = since_the_epoch.as_secs() as i64 * 1000i64
        + (since_the_epoch.subsec_nanos() as i64 / 1_000_000) as i64;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisBlockBuilder {
    pub timestamp: u64,
    pub prevhash: String,
}

impl GenesisBlockBuilder {
    pub fn new() -> Self {
        Self {
            timestamp: 0,
            prevhash: PRE_HASH.to_string(),
        }
    }
    pub fn timestamp(&mut self, timestamp: u64) -> &mut GenesisBlockBuilder {
        self.timestamp = timestamp;
        self
    }

    pub fn prevhash(&mut self, prevhash: String) -> &mut GenesisBlockBuilder {
        self.prevhash = prevhash;
        self
    }

    pub fn build(&self) -> GenesisBlock {
        GenesisBlock {
            timestamp: self.timestamp,
            prevhash: self.prevhash.clone(),
        }
    }
}

#[cfg(test)]
mod controller_test {
    use super::*;
    use crate::util::write_to_file;
    use toml::Value;

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
            prevhash: "0x0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
        };

        genesis.write("example");
    }
}
