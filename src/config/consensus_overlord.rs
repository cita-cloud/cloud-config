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

use crate::constant::{CONSENSUS, CONSENSUS_OVERLORD};
use crate::traits::{TomlWriter, YmlWriter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ConsensusOverlord {
    pub controller_port: u16,

    pub consensus_port: u16,

    pub network_port: u16,

    pub metrics_port: u16,

    pub enable_metrics: bool,
}

impl ConsensusOverlord {
    pub fn new(
        controller_port: u16,
        consensus_port: u16,
        network_port: u16,
        metrics_port: u16,
        enable_metrics: bool,
    ) -> Self {
        Self {
            controller_port,
            consensus_port,
            network_port,
            metrics_port,
            enable_metrics,
        }
    }
}

impl YmlWriter for ConsensusOverlord {
    fn service(&self) -> String {
        CONSENSUS.to_string()
    }
}

impl TomlWriter for ConsensusOverlord {
    fn section(&self) -> String {
        CONSENSUS_OVERLORD.to_string()
    }
}
