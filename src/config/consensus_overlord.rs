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

    pub kms_port: u16,

    pub node_address: String,
}

impl ConsensusOverlord {
    pub fn new(
        controller_port: u16,
        consensus_port: u16,
        network_port: u16,
        kms_port: u16,
        node_address: String,
    ) -> Self {
        Self {
            controller_port,
            consensus_port,
            network_port,
            kms_port,
            node_address,
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
