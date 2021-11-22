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

use crate::config::controller::{GenesisBlock, SystemConfigFile};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeNetworkAddress {
    pub host: String,
    pub port: u16,
    pub domain: String,
}

impl NodeNetworkAddress {
    pub fn new() -> NodeNetworkAddressBuilder {
        NodeNetworkAddressBuilder {
            host: "localhost".to_string(),
            port: 0,
            domain: "".to_string(),
        }
    }
}

pub struct NodeNetworkAddressBuilder {
    pub host: String,
    pub port: u16,
    pub domain: String,
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

    pub fn build(&self) -> NodeNetworkAddress {
        NodeNetworkAddress {
            host: self.host.clone(),
            port: self.port,
            domain: self.domain.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MicroService {
    pub image: String,
    pub tag: String,
}

impl MicroService {
    pub fn new() -> MicroServiceBuilder {
        MicroServiceBuilder {
            image: "".to_string(),
            tag: "latest".to_string(),
        }
    }
}

pub struct MicroServiceBuilder {
    pub image: String,
    pub tag: String,
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

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ChainConfig {
    pub system_config: SystemConfigFile,
    pub genesis_block: GenesisBlock,
    pub node_network_address_list: Vec<NodeNetworkAddress>,
    pub micro_service_list: Vec<MicroService>,
    pub ca_cert_pem: Option<String>,
}

impl ChainConfig {
    pub fn new() -> ChainConfigBuilder {
        ChainConfigBuilder {
            system_config: SystemConfigFile::new().build(),
            genesis_block: GenesisBlock::new().build(),
            node_network_address_list: Vec::new(),
            micro_service_list: Vec::new(),
            ca_cert_pem: None,
        }
    }

    pub fn set_admin(&mut self, admin: String) {
        self.system_config.set_admin(admin);
    }

    pub fn set_validators(&mut self, validators: Vec<String>) {
        self.system_config.set_validators(validators);
    }

    pub fn set_node_network_address_list(&mut self, node_list: Vec<NodeNetworkAddress>) {
        self.node_network_address_list = node_list;
    }

    pub fn set_ca_cert_pem(&mut self, ca_cert_pem: String) {
        self.ca_cert_pem = Some(ca_cert_pem);
    }
}

pub struct ChainConfigBuilder {
    pub system_config: SystemConfigFile,
    pub genesis_block: GenesisBlock,
    pub node_network_address_list: Vec<NodeNetworkAddress>,
    pub micro_service_list: Vec<MicroService>,
    pub ca_cert_pem: Option<String>,
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

    pub fn ca_cert_pem(&mut self, ca_cert_pem: String) -> &mut ChainConfigBuilder {
        self.ca_cert_pem = Some(ca_cert_pem);
        self
    }

    pub fn build(&self) -> ChainConfig {
        ChainConfig {
            system_config: self.system_config.clone(),
            genesis_block: self.genesis_block.clone(),
            node_network_address_list: self.node_network_address_list.clone(),
            micro_service_list: self.micro_service_list.clone(),
            ca_cert_pem: self.ca_cert_pem.clone(),
        }
    }
}
