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

use crate::util::check_address;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct GrpcPorts {
    pub network_port: u16,
    pub consensus_port: u16,
    pub executor_port: u16,
    pub storage_port: u16,
    pub controller_port: u16,
    pub crypto_port: u16,
}

pub struct GrpcPortsBuilder {
    pub network_port: u16,
    pub consensus_port: u16,
    pub executor_port: u16,
    pub storage_port: u16,
    pub controller_port: u16,
    pub crypto_port: u16,
}

impl GrpcPortsBuilder {
    pub fn new() -> Self {
        Self {
            network_port: 50000,
            consensus_port: 50001,
            executor_port: 50002,
            storage_port: 50003,
            controller_port: 50004,
            crypto_port: 50005,
        }
    }
    pub fn network_port(&mut self, network_port: u16) -> &mut GrpcPortsBuilder {
        self.network_port = network_port;
        self
    }

    pub fn consensus_port(&mut self, consensus_port: u16) -> &mut GrpcPortsBuilder {
        self.consensus_port = consensus_port;
        self
    }

    pub fn executor_port(&mut self, executor_port: u16) -> &mut GrpcPortsBuilder {
        self.executor_port = executor_port;
        self
    }

    pub fn storage_port(&mut self, storage_port: u16) -> &mut GrpcPortsBuilder {
        self.storage_port = storage_port;
        self
    }

    pub fn controller_port(&mut self, controller_port: u16) -> &mut GrpcPortsBuilder {
        self.controller_port = controller_port;
        self
    }

    pub fn crypto_port(&mut self, crypto_port: u16) -> &mut GrpcPortsBuilder {
        self.crypto_port = crypto_port;
        self
    }

    pub fn build(&self) -> GrpcPorts {
        GrpcPorts {
            network_port: self.network_port,
            consensus_port: self.consensus_port,
            executor_port: self.executor_port,
            storage_port: self.storage_port,
            controller_port: self.controller_port,
            crypto_port: self.crypto_port,
        }
    }
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct NodeConfig {
    pub grpc_ports: GrpcPorts,
    pub network_listen_port: u16,
    pub log_level: String,
    pub account: String,
}

pub struct NodeConfigBuilder {
    pub grpc_ports: GrpcPorts,
    pub network_listen_port: u16,
    pub log_level: String,
    pub account: String,
}

impl NodeConfigBuilder {
    pub fn new() -> Self {
        Self {
            grpc_ports: GrpcPortsBuilder::new().build(),
            network_listen_port: 40000,
            log_level: "info".to_string(),
            account: "".to_string(),
        }
    }
    pub fn grpc_ports(&mut self, grpc_ports: GrpcPorts) -> &mut NodeConfigBuilder {
        self.grpc_ports = grpc_ports;
        self
    }

    pub fn network_listen_port(&mut self, network_listen_port: u16) -> &mut NodeConfigBuilder {
        self.network_listen_port = network_listen_port;
        self
    }

    pub fn log_level(&mut self, log_level: String) -> &mut NodeConfigBuilder {
        self.log_level = log_level;
        self
    }

    pub fn account(&mut self, account: String) -> &mut NodeConfigBuilder {
        self.account = check_address(&account[..]).to_string();
        self
    }

    pub fn build(&self) -> NodeConfig {
        NodeConfig {
            grpc_ports: self.grpc_ports.clone(),
            network_listen_port: self.network_listen_port,
            log_level: self.log_level.clone(),
            account: self.account.clone(),
        }
    }
}
