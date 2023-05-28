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

use crate::constant::{STORAGE, STORAGE_OPENDAL};
use crate::traits::{TomlWriter, YmlWriter};
use serde::{Deserialize, Serialize};

use super::log_config::LogConfig;
use super::node_config::{CloudStorage, CloudStorageBuilder};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct StorageOpendalConfig {
    pub domain: String,
    pub storage_port: u16,
    pub metrics_port: u16,
    pub enable_metrics: bool,
    pub log_config: LogConfig,
    // cloud storage
    pub cloud_storage: CloudStorage,
}

impl StorageOpendalConfig {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        domain: String,
        storage_port: u16,
        metrics_port: u16,
        enable_metrics: bool,
        log_config: LogConfig,
        access_key_id: String,
        secret_access_key: String,
        endpoint: String,
        bucket: String,
    ) -> Self {
        Self {
            domain,
            storage_port,
            metrics_port,
            enable_metrics,
            log_config,
            cloud_storage: CloudStorageBuilder::default()
                .access_key_id(access_key_id)
                .secret_access_key(secret_access_key)
                .endpoint(endpoint)
                .bucket(bucket)
                .build(),
        }
    }
}

impl TomlWriter for StorageOpendalConfig {
    fn section(&self) -> String {
        STORAGE_OPENDAL.to_string()
    }
}

impl YmlWriter for StorageOpendalConfig {
    fn service(&self) -> String {
        STORAGE.to_string()
    }
}
