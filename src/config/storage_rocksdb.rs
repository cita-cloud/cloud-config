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

use crate::constant::{STORAGE, STORAGE_ROCKSDB};
use crate::traits::{TomlWriter, YmlWriter};
use serde::{Deserialize, Serialize};

use super::log_config::LogConfig;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct StorageRocksdbConfig {
    pub domain: String,
    pub crypto_port: u16,
    pub storage_port: u16,
    pub metrics_port: u16,
    pub enable_metrics: bool,
    pub log_config: LogConfig,
}

impl StorageRocksdbConfig {
    pub fn new(
        domain: String,
        crypto_port: u16,
        storage_port: u16,
        metrics_port: u16,
        enable_metrics: bool,
        log_config: LogConfig,
    ) -> Self {
        Self {
            domain,
            crypto_port,
            storage_port,
            metrics_port,
            enable_metrics,
            log_config,
        }
    }
}

impl TomlWriter for StorageRocksdbConfig {
    fn section(&self) -> String {
        STORAGE_ROCKSDB.to_string()
    }
}

impl YmlWriter for StorageRocksdbConfig {
    fn service(&self) -> String {
        STORAGE.to_string()
    }
}
