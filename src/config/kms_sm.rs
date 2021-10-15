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

use status_code::StatusCode;
use serde_derive::Serialize;
use std::path;

#[derive(Debug, Serialize, Clone)]
pub struct KmsConfig {
    pub kms_port: u16,
}

impl KmsConfig {
    fn write_to_file(&self, path: impl AsRef<path::Path>) {
        crate::util::write_to_file(&self, path, "kms_sm".to_string());
    }
}

pub struct Kms(kms_sm::kms::Kms);

impl crate::traits::Kms for Kms {
    fn create_kms_db(db_path: String, password: String) -> Self {
        Kms(kms_sm::kms::Kms::new(db_path, password))
    }

    fn generate_key_pair(&self, description: String) -> Result<(u64, Vec<u8>), StatusCode> {
        self.0.generate_key_pair(description)
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

        config.write_to_file("example/config.toml");
    }
}
