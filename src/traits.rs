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
use crate::config::kms_sm::KmsSmConfig;
use crate::config::network_p2p::NetConfig;
use crate::config::network_tls::NetworkConfig;
use crate::config::storage_rocksdb::StorageRocksdbConfig;
use crate::util;
use serde::{Deserialize, Serialize};
use std::{fs, path};
use crate::constant::LOG4RS_YAML;

pub trait Kms {
    fn create_kms_db(db_path: String, password: String) -> Self;
    fn generate_key_pair(&self, description: String) -> (u64, Vec<u8>);
}

pub trait TomlWriter {
    fn write(&self, path: impl AsRef<path::Path>)
    where
        Self: Serialize,
    {
        util::write_to_file(&self, path, self.section())
    }

    fn section(&self) -> String;
}

pub trait YmlWriter {
    fn service(&self) -> String;

    fn write_log4rs(&self, path: &str, is_stdout: bool)
    where
        Self: Serialize,
    {
        let service = self.service();
        fs::write(
            format!("{}/{}-{}", path, service, LOG4RS_YAML),
            format!(
                r#"# Scan this file for changes every 30 seconds
refresh_rate: 30 seconds

appenders:
# An appender named "stdout" that writes to stdout
  stdout:
    kind: console

  journey-service:
    kind: rolling_file
    path: "logs/{}-service.log"
    policy:
      # Identifies which policy is to be used. If no kind is specified, it will
      # default to "compound".
      kind: compound
      # The remainder of the configuration is passed along to the policy's
      # deserializer, and will vary based on the kind of policy.
      trigger:
        kind: size
        limit: 50mb
      roller:
        kind: fixed_window
        base: 1
        count: 5
        pattern: "logs/{}-service.{{}}.gz"

# Set the default logging level and attach the default appender to the root
root:
  level: {}
  appenders:
    - {}
"#,
                service,
                service,
                "info",
                if is_stdout {
                    "stdout"
                } else {
                    "journey-service"
                }
            ),
        )
        .unwrap();
    }
}

pub trait Writer: TomlWriter + YmlWriter {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateConfig {
    pub system_config: SystemConfigFile,
    pub genesis_block: GenesisBlock,
    pub network_p2p: Option<NetConfig>,
    pub network_tls: Option<NetworkConfig>,
    pub controller: Option<ControllerConfig>,
    pub kms_sm: Option<KmsSmConfig>,
    pub storage_rocksdb: Option<StorageRocksdbConfig>,
    pub executor_evm: Option<ExecutorEvmConfig>,
    pub consensus_raft: Option<Consensus>,
}
