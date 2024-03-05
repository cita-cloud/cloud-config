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

use crate::config::controller::{
    GenesisBlock, GenesisBlockBuilder, SystemConfigBuilder, SystemConfigFile,
};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct NodeNetworkAddress {
    pub host: String,
    pub port: u16,
    pub domain: String,
    pub cluster: String,
    pub name_space: String,
}

impl PartialEq for NodeNetworkAddress {
    fn eq(&self, other: &Self) -> bool {
        self.domain == other.domain
    }
}

impl Hash for NodeNetworkAddress {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.domain.hash(state);
        self.port.hash(state);
        self.host.hash(state);
    }
}

pub struct NodeNetworkAddressBuilder {
    pub host: String,
    pub port: u16,
    pub domain: String,
    pub cluster: String,
    pub name_space: String,
}

impl Default for NodeNetworkAddressBuilder {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 40000,
            domain: "".to_string(),
            cluster: "".to_string(),
            name_space: "default".to_string(),
        }
    }
}

impl NodeNetworkAddressBuilder {
    pub fn host(&mut self, host: String) -> &mut NodeNetworkAddressBuilder {
        self.host = host;
        self
    }

    pub fn port(&mut self, port: u16) -> &mut NodeNetworkAddressBuilder {
        self.port = port;
        self
    }

    pub fn domain(&mut self, domain: String) -> &mut NodeNetworkAddressBuilder {
        self.domain = domain;
        self
    }

    pub fn cluster(&mut self, cluster: String) -> &mut NodeNetworkAddressBuilder {
        self.cluster = cluster;
        self
    }

    #[allow(dead_code)]
    pub fn name_space(&mut self, name_space: String) -> &mut NodeNetworkAddressBuilder {
        self.name_space = name_space;
        self
    }

    pub fn build(&self) -> NodeNetworkAddress {
        NodeNetworkAddress {
            host: self.host.clone(),
            port: self.port,
            domain: self.domain.clone(),
            cluster: self.cluster.clone(),
            name_space: self.name_space.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MicroService {
    pub image: String,
    pub tag: String,
}

pub struct MicroServiceBuilder {
    pub image: String,
    pub tag: String,
}

impl Default for MicroServiceBuilder {
    fn default() -> Self {
        MicroServiceBuilder {
            image: "".to_string(),
            tag: "latest".to_string(),
        }
    }
}

impl MicroServiceBuilder {
    pub fn image(&mut self, image: String) -> &mut MicroServiceBuilder {
        self.image = image;
        self
    }

    pub fn tag(&mut self, tag: String) -> &mut MicroServiceBuilder {
        self.tag = tag;
        self
    }

    pub fn build(&self) -> MicroService {
        MicroService {
            image: self.image.clone(),
            tag: self.tag.clone(),
        }
    }
}

#[derive(Debug, Serialize, Clone, Deserialize, Eq, PartialEq)]
pub enum ConfigStage {
    Init,
    Public,
    Finalize,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ChainConfig {
    pub system_config: SystemConfigFile,
    pub genesis_block: GenesisBlock,
    pub node_network_address_list: Vec<NodeNetworkAddress>,
    pub micro_service_list: Vec<MicroService>,
    pub stage: ConfigStage,
}

impl ChainConfig {
    pub fn set_admin(&mut self, admin: String) {
        self.system_config.set_admin(admin);
    }

    pub fn set_validators(&mut self, validators: Vec<String>) {
        self.system_config.set_validators(validators);
    }

    pub fn set_node_network_address_list(&mut self, node_list: Vec<NodeNetworkAddress>) {
        self.node_network_address_list = node_list;
    }

    pub fn set_stage(&mut self, stage: ConfigStage) {
        self.stage = stage;
    }
}

pub struct ChainConfigBuilder {
    pub system_config: SystemConfigFile,
    pub genesis_block: GenesisBlock,
    pub node_network_address_list: Vec<NodeNetworkAddress>,
    pub micro_service_list: Vec<MicroService>,
    pub stage: ConfigStage,
}

impl Default for ChainConfigBuilder {
    fn default() -> Self {
        Self {
            system_config: SystemConfigBuilder::default().build(),
            genesis_block: GenesisBlockBuilder::default().build(),
            node_network_address_list: Vec::new(),
            micro_service_list: Vec::new(),
            stage: ConfigStage::Init,
        }
    }
}

impl ChainConfigBuilder {
    pub fn system_config(&mut self, system_config: SystemConfigFile) -> &mut ChainConfigBuilder {
        self.system_config = system_config;
        self
    }

    pub fn genesis_block(&mut self, genesis_block: GenesisBlock) -> &mut ChainConfigBuilder {
        self.genesis_block = genesis_block;
        self
    }

    #[allow(dead_code)]
    pub fn node_network_address_list(
        &mut self,
        node_network_address_list: Vec<NodeNetworkAddress>,
    ) -> &mut ChainConfigBuilder {
        self.node_network_address_list = node_network_address_list;
        self
    }

    pub fn micro_service_list(
        &mut self,
        micro_service_list: Vec<MicroService>,
    ) -> &mut ChainConfigBuilder {
        self.micro_service_list = micro_service_list;
        self
    }

    pub fn build(&self) -> ChainConfig {
        ChainConfig {
            system_config: self.system_config.clone(),
            genesis_block: self.genesis_block.clone(),
            node_network_address_list: self.node_network_address_list.clone(),
            micro_service_list: self.micro_service_list.clone(),
            stage: self.stage.clone(),
        }
    }
}
