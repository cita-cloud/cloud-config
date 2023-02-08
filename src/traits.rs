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

use crate::config::consensus_raft::Consensus;
use crate::config::controller::{ControllerConfig, GenesisBlock, SystemConfigFile};
use crate::config::executor_evm::ExecutorEvmConfig;
use crate::config::network_zenoh::ZenohConfig;
use crate::config::storage_rocksdb::StorageRocksdbConfig;
use crate::util;
use serde::{Deserialize, Serialize};
use std::path;

pub trait TomlWriter {
    fn write(&self, path: impl AsRef<path::Path>)
    where
        Self: Serialize,
    {
        util::write_to_file(self, path, self.section())
    }

    fn section(&self) -> String;
}

pub trait YmlWriter {
    fn service(&self) -> String;
}

pub trait Writer: TomlWriter + YmlWriter {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateConfig {
    pub system_config: SystemConfigFile,
    pub genesis_block: GenesisBlock,
    pub network_zenoh: Option<ZenohConfig>,
    pub controller: Option<ControllerConfig>,
    pub storage_rocksdb: Option<StorageRocksdbConfig>,
    pub executor_evm: Option<ExecutorEvmConfig>,
    pub consensus_raft: Option<Consensus>,
}
