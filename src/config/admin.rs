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
use crate::config::network_p2p::PeerConfig;
use crate::config::network_tls::PeerConfig as TlsConfig;
use crate::constant::{ADMIN_CONFIG, CURRENT_CONFIG};
use crate::traits::TomlWriter;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct AdminConfig {
    pub db_key: Option<String>,

    pub key_id: u64,

    pub admin_address: String,

    pub admin_password: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CurrentConfig {
    pub count: u16,

    pub peers: Option<Vec<PeerConfig>>,

    pub tls_peers: Option<Vec<crate::config::network_tls::PeerConfig>>,

    pub addresses: Vec<String>,

    pub rpc_ports: Vec<u16>,

    pub p2p_ports: Vec<u16>,

    pub ips: Vec<String>,

    pub ca_cert_pem: String,

    pub ca_key_pem: String,

    pub use_num: bool,

    pub history_addresses: Vec<String>,
}

pub struct AdminParam {
    pub admin_key: u64,
    pub admin_address: String,
    pub chain_path: String,
    pub key_ids: Vec<u64>,
    pub addresses: Vec<String>,
    pub uris: Option<Vec<PeerConfig>>,
    pub tls_peers: Option<Vec<TlsConfig>>,
    pub ca_cert_pem: String,
    pub ca_key_pem: String,
    pub genesis: GenesisBlock,
    pub system: SystemConfigFile,
    pub rpc_ports: Vec<u16>,
    pub p2p_ports: Vec<u16>,
    pub ips: Vec<String>,
    pub count: u16,
    pub use_num: bool,
    pub exist_addressses: Vec<String>,
}

impl CurrentConfig {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        count: u16,
        peers: &[PeerConfig],
        tls_peers: Vec<TlsConfig>,
        addresses: Vec<String>,
        rpc_ports: Vec<u16>,
        p2p_ports: Vec<u16>,
        ips: Vec<String>,
        ca_cert_pem: String,
        ca_key_pem: String,
        use_num: bool,
        history_addresses: Vec<String>,
    ) -> Self {
        Self {
            count,
            peers: Some(peers.to_owned()),
            tls_peers: Some(tls_peers),
            addresses,
            rpc_ports,
            p2p_ports,
            ips,
            ca_cert_pem,
            ca_key_pem,
            use_num,
            history_addresses,
        }
    }
}
impl TomlWriter for CurrentConfig {
    fn section(&self) -> String {
        CURRENT_CONFIG.to_string()
    }
}

impl AdminConfig {
    pub fn new(key_id: u64, admin_address: String, admin_password: String) -> Self {
        Self {
            db_key: None,
            key_id,
            admin_address,
            admin_password,
        }
    }
}

impl TomlWriter for AdminConfig {
    fn section(&self) -> String {
        ADMIN_CONFIG.to_string()
    }
}
