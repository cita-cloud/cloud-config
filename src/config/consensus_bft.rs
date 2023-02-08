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

use crate::constant::{CONSENSUS, CONSENSUS_BFT};
use crate::traits::{TomlWriter, YmlWriter};
use serde::{Deserialize, Serialize};

use super::log_config::LogConfig;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ConsensusBft {
    pub domain: String,

    pub controller_port: u16,

    pub consensus_port: u16,

    pub network_port: u16,

    pub crypto_port: u16,

    pub node_address: String,

    pub metrics_port: u16,

    pub enable_metrics: bool,

    pub log_config: LogConfig,
}

impl ConsensusBft {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        domain: String,
        controller_port: u16,
        consensus_port: u16,
        network_port: u16,
        crypto_port: u16,
        node_address: String,
        metrics_port: u16,
        enable_metrics: bool,
        log_config: LogConfig,
    ) -> Self {
        Self {
            domain,
            controller_port,
            consensus_port,
            network_port,
            crypto_port,
            node_address,
            metrics_port,
            enable_metrics,
            log_config,
        }
    }
}

impl YmlWriter for ConsensusBft {
    fn service(&self) -> String {
        CONSENSUS.to_string()
    }
}

impl TomlWriter for ConsensusBft {
    fn section(&self) -> String {
        CONSENSUS_BFT.to_string()
    }
}
