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

// use status_code::StatusCode;
use serde::{Deserialize, Serialize};
use std::path;
use crate::constant::KMS_SM;
use crate::traits::TomlWriter;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct KmsConfig {
    pub kms_port: u16,
}

impl KmsConfig {
    pub fn new(kms_port: u16) -> Self {
        Self {
            kms_port
        }
    }
}

impl TomlWriter for KmsConfig {
    fn section(&self) -> String {
        KMS_SM.to_string()
    }
}

pub struct Kms(kms_sm::kms::Kms);

impl crate::traits::Kms for Kms {
    fn create_kms_db(db_path: String, password: String) -> Self {
        Kms(kms_sm::kms::Kms::new(db_path, password))
    }

    fn generate_key_pair(&self, description: String) -> (u64, Vec<u8>) {
        self.0.generate_key_pair(description).unwrap()
    }
}

#[cfg(test)]
mod kms_test {
    use super::*;
    use toml::Value;
    use crate::util::write_to_file;

    #[test]
    fn basic_test() {
        let _ = std::fs::remove_file("example/config.toml");

        let config = KmsConfig {
            kms_port: 51235,
        };

        config.write("example/config.toml");
    }
}
