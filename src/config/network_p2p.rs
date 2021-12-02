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

use crate::constant::{NETWORK, NETWORK_P2P};
use crate::traits::{TomlWriter, YmlWriter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct NetConfig {
    pub port: Option<u16>,

    pub grpc_port: Option<u16>,

    #[serde(default)]
    // https://github.com/alexcrichton/toml-rs/issues/258
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub peers: Vec<PeerConfig>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct PeerConfig {
    pub address: String,
}

impl NetConfig {
    pub fn new(port: u16, grpc_port: u16, addresses: &[PeerConfig]) -> Self {
        let mut peers = Vec::with_capacity(addresses.len());
        for address in addresses {
            peers.push(address.clone());
        }
        Self {
            port: Some(port),
            grpc_port: Some(grpc_port),
            peers,
        }
    }
}

impl TomlWriter for NetConfig {
    fn section(&self) -> String {
        NETWORK_P2P.to_string()
    }
}

impl YmlWriter for NetConfig {
    fn service(&self) -> String {
        NETWORK.to_string()
    }
}

#[cfg(test)]
mod network_p2p_test {
    use super::*;
    use crate::util::write_to_file;
    use toml::Value;

    #[test]
    fn basic_test() {
        // let _ = std::fs::remove_file("example/config.toml");
        //
        // let peers: &Vec<String> =  &vec!["/ip4/127.0.0.1/tcp/40001".to_string(), "/ip4/127.0.0.1/tcp/40002".to_string(), "/ip4/127.0.0.1/tcp/40003".to_string()];
        // let config = NetConfig::new(
        //     51230,
        //     40000,
        //     peers,
        // );
        //
        // config.write("example");
    }
}
