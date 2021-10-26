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

use serde::{Deserialize, Serialize};
use std::path;
use crate::config::network_p2p::PeerConfig;
use crate::constant::{ADMIN_CONFIG, CURRENT_CONFIG};
use crate::traits::TomlWriter;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct AdminConfig {
    pub db_key: Option<String>,

    pub key_id: u64,

    pub db_path: Option<String>,

    pub admin_address: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CurrentConfig {
    pub peers: Vec<PeerConfig>,

    pub tls_peers: Vec<crate::config::network_tls::PeerConfig>,

    pub addresses: Vec<String>,
}

impl CurrentConfig {
    pub fn new(peers: &Vec<String>,
               tls_peers: Vec<crate::config::network_tls::PeerConfig>,
               addresses: Vec<String>,) -> Self {
        let mut peers_new = Vec::with_capacity(peers.len());
        for peer in peers {
            peers_new.push(PeerConfig{
                address: peer.clone(),
            })
        }
        Self {
            peers: peers_new,
            tls_peers,
            addresses,
        }
    }
}
impl TomlWriter for CurrentConfig{
    fn section(&self) -> String {
        CURRENT_CONFIG.to_string()
    }
}


impl AdminConfig {
    pub fn new(db_key: String, key_id: u64, db_path: String, admin_address: String) -> Self {
        Self {
            db_key: Some(db_key),
            key_id,
            db_path: Some(db_path),
            admin_address,
        }
    }

    pub fn default(key_id: u64, admin_address: String) -> Self {
        Self {
            db_key: None,
            key_id,
            db_path: None,
            admin_address,
        }
    }
}

impl TomlWriter for AdminConfig{
    fn section(&self) -> String {
        ADMIN_CONFIG.to_string()
    }
}

