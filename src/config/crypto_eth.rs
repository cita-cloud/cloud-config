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

use crate::constant::{CRYPTO, CRYPTO_ETH};
use crate::traits::{TomlWriter, YmlWriter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CryptoEthConfig {
    pub crypto_port: u16,
}

impl CryptoEthConfig {
    pub fn new(crypto_port: u16) -> Self {
        Self { crypto_port }
    }
}
impl TomlWriter for CryptoEthConfig {
    fn section(&self) -> String {
        CRYPTO_ETH.to_string()
    }
}

impl YmlWriter for CryptoEthConfig {
    fn service(&self) -> String {
        CRYPTO.to_string()
    }
}
