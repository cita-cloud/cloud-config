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

use crate::constant::CONSENSUS_RAFT;
use crate::traits::TomlWriter;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Consensus {
    pub network_port: u16,

    pub controller_port: u16,

    pub node_addr: String,

    pub grpc_listen_port: u16,

    pub metrics_port: u16,

    pub enable_metrics: bool,

    pub log_level: String,
}

impl Consensus {
    pub fn new(
        network_port: u16,
        controller_port: u16,
        node_addr: String,
        grpc_listen_port: u16,
        metrics_port: u16,
        enable_metrics: bool,
        log_level: String,
    ) -> Self {
        Self {
            network_port,
            controller_port,
            node_addr,
            grpc_listen_port,
            metrics_port,
            enable_metrics,
            log_level,
        }
    }
}

impl TomlWriter for Consensus {
    fn section(&self) -> String {
        CONSENSUS_RAFT.to_string()
    }
}
