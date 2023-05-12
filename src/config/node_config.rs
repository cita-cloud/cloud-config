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
}

pub struct GrpcPortsBuilder {
    pub network_port: u16,
    pub consensus_port: u16,
    pub executor_port: u16,
    pub storage_port: u16,
    pub controller_port: u16,
}

impl Default for GrpcPortsBuilder {
    fn default() -> Self {
        Self {
            network_port: 50000,
            consensus_port: 50001,
            executor_port: 50002,
            storage_port: 50003,
            controller_port: 50004,
        }
    }
}

impl GrpcPortsBuilder {
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

    pub fn build(&self) -> GrpcPorts {
        GrpcPorts {
            network_port: self.network_port,
            consensus_port: self.consensus_port,
            executor_port: self.executor_port,
            storage_port: self.storage_port,
            controller_port: self.controller_port,
        }
    }
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct MetricsPorts {
    pub network_metrics_port: u16,
    pub consensus_metrics_port: u16,
    pub executor_metrics_port: u16,
    pub storage_metrics_port: u16,
    pub controller_metrics_port: u16,
}

pub struct MetricsPortsBuilder {
    pub network_metrics_port: u16,
    pub consensus_metrics_port: u16,
    pub executor_metrics_port: u16,
    pub storage_metrics_port: u16,
    pub controller_metrics_port: u16,
}

impl Default for MetricsPortsBuilder {
    fn default() -> Self {
        Self {
            network_metrics_port: 60000,
            consensus_metrics_port: 60001,
            executor_metrics_port: 60002,
            storage_metrics_port: 60003,
            controller_metrics_port: 60004,
        }
    }
}

impl MetricsPortsBuilder {
    pub fn network_metrics_port(&mut self, network_metrics_port: u16) -> &mut MetricsPortsBuilder {
        self.network_metrics_port = network_metrics_port;
        self
    }

    pub fn consensus_metrics_port(
        &mut self,
        consensus_metrics_port: u16,
    ) -> &mut MetricsPortsBuilder {
        self.consensus_metrics_port = consensus_metrics_port;
        self
    }

    pub fn executor_metrics_port(
        &mut self,
        executor_metrics_port: u16,
    ) -> &mut MetricsPortsBuilder {
        self.executor_metrics_port = executor_metrics_port;
        self
    }

    pub fn storage_metrics_port(&mut self, storage_metrics_port: u16) -> &mut MetricsPortsBuilder {
        self.storage_metrics_port = storage_metrics_port;
        self
    }

    pub fn controller_metrics_port(
        &mut self,
        controller_metrics_port: u16,
    ) -> &mut MetricsPortsBuilder {
        self.controller_metrics_port = controller_metrics_port;
        self
    }

    pub fn build(&self) -> MetricsPorts {
        MetricsPorts {
            network_metrics_port: self.network_metrics_port,
            consensus_metrics_port: self.consensus_metrics_port,
            executor_metrics_port: self.executor_metrics_port,
            storage_metrics_port: self.storage_metrics_port,
            controller_metrics_port: self.controller_metrics_port,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudStorage {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub endpoint: String,
    pub bucket: String,
}

pub struct CloudStorageBuilder {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub endpoint: String,
    pub bucket: String,
}

impl Default for CloudStorageBuilder {
    fn default() -> Self {
        Self {
            access_key_id: "".to_string(),
            secret_access_key: "".to_string(),
            endpoint: "".to_string(),
            bucket: "".to_string(),
        }
    }
}

impl CloudStorageBuilder {
    pub fn access_key_id(&mut self, access_key_id: String) -> &mut CloudStorageBuilder {
        self.access_key_id = access_key_id;
        self
    }

    pub fn secret_access_key(&mut self, secret_access_key: String) -> &mut CloudStorageBuilder {
        self.secret_access_key = secret_access_key;
        self
    }

    pub fn endpoint(&mut self, endpoint: String) -> &mut CloudStorageBuilder {
        self.endpoint = endpoint;
        self
    }

    pub fn bucket(&mut self, bucket: String) -> &mut CloudStorageBuilder {
        self.bucket = bucket;
        self
    }

    pub fn build(&self) -> CloudStorage {
        CloudStorage {
            access_key_id: self.access_key_id.clone(),
            secret_access_key: self.secret_access_key.clone(),
            endpoint: self.endpoint.clone(),
            bucket: self.bucket.clone(),
        }
    }
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct NodeConfig {
    pub grpc_ports: GrpcPorts,
    pub metrics_ports: MetricsPorts,
    pub network_listen_port: u16,
    pub log_level: String,
    pub log_file_path: Option<String>,
    pub jaeger_agent_endpoint: Option<String>,
    pub account: String,
    pub enable_metrics: bool,
    pub is_danger: bool,
    pub cloud_storage: CloudStorage,
}

pub struct NodeConfigBuilder {
    pub grpc_ports: GrpcPorts,
    pub metrics_ports: MetricsPorts,
    pub network_listen_port: u16,
    pub log_level: String,
    pub log_file_path: Option<String>,
    pub jaeger_agent_endpoint: Option<String>,
    pub account: String,
    pub enable_metrics: bool,
    pub is_danger: bool,
    pub cloud_storage: CloudStorage,
}

impl Default for NodeConfigBuilder {
    fn default() -> Self {
        Self {
            grpc_ports: GrpcPortsBuilder::default().build(),
            metrics_ports: MetricsPortsBuilder::default().build(),
            network_listen_port: 40000,
            log_level: "info".to_string(),
            log_file_path: None,
            jaeger_agent_endpoint: None,
            account: "".to_string(),
            enable_metrics: true,
            is_danger: false,
            cloud_storage: CloudStorageBuilder::default().build(),
        }
    }
}

impl NodeConfigBuilder {
    pub fn grpc_ports(&mut self, grpc_ports: GrpcPorts) -> &mut NodeConfigBuilder {
        self.grpc_ports = grpc_ports;
        self
    }

    pub fn metrics_ports(&mut self, metrics_ports: MetricsPorts) -> &mut NodeConfigBuilder {
        self.metrics_ports = metrics_ports;
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

    pub fn enable_metrics(&mut self, enable_metrics: bool) -> &mut NodeConfigBuilder {
        self.enable_metrics = enable_metrics;
        self
    }

    pub fn log_file_path(&mut self, log_file_path: Option<String>) -> &mut NodeConfigBuilder {
        self.log_file_path = log_file_path;
        self
    }

    pub fn jaeger_agent_endpoint(
        &mut self,
        jaeger_agent_endpoint: Option<String>,
    ) -> &mut NodeConfigBuilder {
        self.jaeger_agent_endpoint = jaeger_agent_endpoint;
        self
    }

    pub fn is_danger(&mut self, is_danger: bool) -> &mut NodeConfigBuilder {
        self.is_danger = is_danger;
        self
    }

    pub fn cloud_storage(&mut self, cloud_storage: CloudStorage) -> &mut NodeConfigBuilder {
        self.cloud_storage = cloud_storage;
        self
    }

    pub fn build(&self) -> NodeConfig {
        NodeConfig {
            grpc_ports: self.grpc_ports.clone(),
            metrics_ports: self.metrics_ports.clone(),
            network_listen_port: self.network_listen_port,
            log_level: self.log_level.clone(),
            log_file_path: self.log_file_path.clone(),
            jaeger_agent_endpoint: self.jaeger_agent_endpoint.clone(),
            account: self.account.clone(),
            enable_metrics: self.enable_metrics,
            is_danger: self.is_danger,
            cloud_storage: self.cloud_storage.clone(),
        }
    }
}
