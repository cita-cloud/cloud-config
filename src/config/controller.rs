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
    CONTROLLER, DEFAULT_BLOCK_INTERVAL, DEFAULT_BLOCK_LIMIT, DEFAULT_QUOTA_LIMIT, GENESIS_BLOCK,
    PRE_HASH, SYSTEM_CONFIG,
};
use crate::traits::{TomlWriter, YmlWriter};
use crate::util::check_address;
use serde::{Deserialize, Serialize};

use super::log_config::LogConfig;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ControllerConfig {
    pub domain: String,

    pub network_port: u16,

    pub consensus_port: u16,

    pub executor_port: u16,

    pub storage_port: u16,

    pub controller_port: u16,

    pub crypto_port: u16,

    pub node_address: String,

    pub validator_address: String,

    pub metrics_port: u16,

    pub enable_metrics: bool,

    pub log_config: LogConfig,

    pub is_danger: bool,
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
    pub quota_limit: u64,
}

impl SystemConfigFile {
    pub fn set_admin(&mut self, admin: String) {
        self.admin = check_address(&admin[..]).to_string();
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
    pub quota_limit: u64,
}

impl Default for SystemConfigBuilder {
    fn default() -> Self {
        Self {
            version: 0,
            chain_id: "".to_string(),
            admin: "".to_string(),
            block_interval: DEFAULT_BLOCK_INTERVAL,
            validators: Vec::new(),
            block_limit: DEFAULT_BLOCK_LIMIT,
            quota_limit: DEFAULT_QUOTA_LIMIT,
        }
    }
}

impl SystemConfigBuilder {
    pub fn version(&mut self, version: u32) -> &mut SystemConfigBuilder {
        self.version = version;
        self
    }

    pub fn chain_id(&mut self, chain_id: String) -> &mut SystemConfigBuilder {
        self.chain_id = chain_id;
        self
    }

    #[allow(dead_code)]
    pub fn admin(&mut self, chain_id: String) -> &mut SystemConfigBuilder {
        self.chain_id = chain_id;
        self
    }

    pub fn block_interval(&mut self, block_interval: u32) -> &mut SystemConfigBuilder {
        self.block_interval = block_interval;
        self
    }

    #[allow(dead_code)]
    pub fn validators(&mut self, validators: Vec<String>) -> &mut SystemConfigBuilder {
        self.validators = validators;
        self
    }

    pub fn block_limit(&mut self, block_limit: u64) -> &mut SystemConfigBuilder {
        self.block_limit = block_limit;
        self
    }

    pub fn quota_limit(&mut self, quota_limit: u64) -> &mut SystemConfigBuilder {
        self.quota_limit = quota_limit;
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
            quota_limit: self.quota_limit,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisBlock {
    pub timestamp: u64,
    pub prevhash: String,
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

impl Default for GenesisBlockBuilder {
    fn default() -> Self {
        Self {
            timestamp: 0,
            prevhash: PRE_HASH.to_string(),
        }
    }
}

impl GenesisBlockBuilder {
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

    #[test]
    fn basic_test() {
        let config = ControllerConfig {
            domain: "test-chain-0".into(),
            network_port: 51230,
            consensus_port: 51231,
            executor_port: 51232,
            storage_port: 51233,
            controller_port: 51234,
            crypto_port: 51235,
            node_address: "/mnt/node_address".into(),
            validator_address: "/mnt/validator_address".into(),
            metrics_port: 61234,
            enable_metrics: true,
            log_config: LogConfig::default(),
            is_danger: false,
        };

        config.write("example");

        let genesis = GenesisBlock {
            timestamp: 1633765324292,
            prevhash: "0x0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
        };

        genesis.write("example");

        let _ = std::fs::remove_file("example");
    }
}
