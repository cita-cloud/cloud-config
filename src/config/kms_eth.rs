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

use crate::constant::{KMS, KMS_ETH};
use crate::traits::{TomlWriter, YmlWriter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct KmsEthConfig {
    pub kms_port: u16,
    pub db_key: String,
}

impl KmsEthConfig {
    pub fn new(kms_port: u16, db_key: String) -> Self {
        Self { kms_port, db_key }
    }
}
impl TomlWriter for KmsEthConfig {
    fn section(&self) -> String {
        KMS_ETH.to_string()
    }
}

pub struct KmsEth(kms_eth::kms::Kms);

impl crate::traits::Kms for KmsEth {
    fn create_kms_db(db_path: String, password: String) -> Self {
        KmsEth(kms_eth::kms::Kms::new(db_path, password))
    }

    fn generate_key_pair(&self, description: String) -> (u64, Vec<u8>) {
        self.0.generate_key_pair(description).unwrap()
    }
}

impl YmlWriter for KmsEthConfig {
    fn service(&self) -> String {
        KMS.to_string()
    }
}
