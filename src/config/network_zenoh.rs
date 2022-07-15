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

use crate::constant::{NETWORK, NETWORK_ZENOH};
use crate::traits::{TomlWriter, YmlWriter};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeerConfig {
    pub protocol: String,
    pub port: u16,
    pub domain: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModuleConfig {
    pub module_name: String,
    pub hostname: String,
    pub port: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ZenohConfig {
    pub grpc_port: u16,
    pub domain: String,
    pub protocol: String,
    pub port: u16,

    pub ca_cert: String,

    pub cert: String,

    pub priv_key: String,

    #[serde(default)]
    // https://github.com/alexcrichton/toml-rs/issues/258
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub peers: Vec<PeerConfig>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub modules: Vec<ModuleConfig>,

    pub node_address: String,
    pub validator_address: String,
    pub chain_id: String,
}

impl TomlWriter for ZenohConfig {
    fn section(&self) -> String {
        NETWORK_ZENOH.to_string()
    }
}

impl YmlWriter for ZenohConfig {
    fn service(&self) -> String {
        NETWORK.to_string()
    }
}
