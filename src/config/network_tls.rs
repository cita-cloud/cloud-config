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

use crate::constant::{NETWORK_TLS};
use crate::traits::TomlWriter;
use serde::{Deserialize, Serialize};
fn default_reconnect_timeout() -> Option<u64> {
    Some(5)
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeerConfig {
    pub host: String,
    pub port: u16,

    // TODO: is this name suitable?
    pub domain: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkConfig {
    pub grpc_port: Option<u16>,
    pub listen_port: Option<u16>,

    #[serde(default = "default_reconnect_timeout")]
    pub reconnect_timeout: Option<u64>, // in seconds

    pub ca_cert: Option<String>,

    pub cert: Option<String>,
    // TODO: better security
    pub priv_key: Option<String>,

    #[serde(default)]
    // https://github.com/alexcrichton/toml-rs/issues/258
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub peers: Vec<PeerConfig>,
}
impl NetworkConfig {
    pub fn new(
        listen_port: u16,
        grpc_port: u16,
        ca_cert: String,
        cert: String,
        priv_key: String,
        peers: Vec<PeerConfig>,
    ) -> Self {
        Self {
            listen_port: Some(listen_port),
            grpc_port: Some(grpc_port),
            reconnect_timeout: default_reconnect_timeout(),
            ca_cert: Some(ca_cert),
            cert: Some(cert),
            priv_key: Some(priv_key),
            peers,
        }
    }

    pub fn default(
        peers: Vec<PeerConfig>,
    ) -> Self {
        Self {
            listen_port: None,
            grpc_port: None,
            reconnect_timeout: None,
            ca_cert: None,
            cert: None,
            priv_key: None,
            peers,
        }
    }
}

impl TomlWriter for NetworkConfig {
    fn section(&self) -> String {
        NETWORK_TLS.to_string()
    }
}