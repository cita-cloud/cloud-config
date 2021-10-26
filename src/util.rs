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

use std::{path, fs};
use toml::Value;
use std::io::Write;
use toml::de::Error;
use crate::config::admin::{AdminConfig, CurrentConfig};
use crate::config::controller::{ControllerConfig, GenesisBlock, SystemConfigFile};
use crate::config::network_p2p::{NetConfig};
use crate::traits::TomlWriter;
use serde::{Deserialize, Serialize};
use crate::config::executor_evm::ExecutorConfig;
use crate::config::kms_sm::KmsConfig;
use crate::config::network_tls::NetworkConfig;
use crate::config::storage_rocksdb::StorageConfig;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateConfig {
    pub admin_config: AdminConfig,
    pub system_config: SystemConfigFile,
    pub genesis_block: GenesisBlock,
    pub network_p2p: NetConfig,
    pub network_tls: NetworkConfig,
    pub controller: Option<ControllerConfig>,
    pub kms_sm: Option<KmsConfig>,
    pub storage_rocksdb: Option<StorageConfig>,
    pub executor_evm: Option<ExecutorConfig>,
    pub current_config: Option<CurrentConfig>,
}

pub fn write_to_file<T: serde::Serialize>(content: T, path: impl AsRef<path::Path>, name: String) {
    let value = Value::try_from(content).unwrap();
    let mut table = toml::map::Map::new();
    table.insert(name, value);
    let toml = toml::Value::Table(table);

    let mut file = fs::OpenOptions::new().create(true).append(true).open(path.as_ref()).unwrap();
    file.write_all(toml::to_string_pretty(&toml).unwrap().as_bytes()).unwrap();
    file.write(b"\n").unwrap();
}

pub fn write_whole_to_file(content: AggregateConfig, path: impl AsRef<path::Path>) {
    if let Some(t) = content.controller {
        t.write(&path);
    }
    if let Some(t) = content.kms_sm {
        t.write(&path);
    }
    if let Some(t) = content.storage_rocksdb {
        t.write(&path);
    }
    if let Some(t) = content.executor_evm {
        t.write(&path);
    }
    if let Some(t) = content.current_config {
        t.write(&path);
    }

    content.admin_config.write(&path);
    content.system_config.write(&path);
    content.genesis_block.write(&path);
    content.network_p2p.write(&path);
    content.network_tls.write(&path);
}

pub fn read_from_file(path: impl AsRef<path::Path>) -> Result<AggregateConfig, Error> {
    let buffer = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("Error while loading config: [{}]", err));
    toml::from_str::<AggregateConfig>(&buffer)
}

#[cfg(test)]
mod util_test {
    use crate::util::read_from_file;

    #[test]
    fn util_test() {
        let config = read_from_file("cita-chain/config.toml");
        println!("{:?}", config)
    }
}


