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

pub const CONTROLLER: &str = "controller";
#[allow(dead_code)]
pub const CONSENSUS: &str = "consensus";
pub const CONSENSUS_BFT: &str = "consensus_bft";
pub const CONSENSUS_RAFT: &str = "consensus_raft";
pub const GENESIS_BLOCK: &str = "genesis_block";
pub const SYSTEM_CONFIG: &str = "system_config";
pub const NETWORK: &str = "network";
pub const NETWORK_P2P: &str = "network_p2p";
pub const NETWORK_TLS: &str = "network_tls";
pub const KMS_SM: &str = "kms_sm";
pub const KMS_ETH: &str = "kms_eth";
pub const KMS_DB: &str = "kms.db";
pub const KMS: &str = "kms";
pub const STORAGE_ROCKSDB: &str = "storage_rocksdb";
pub const STORAGE: &str = "storage";
pub const EXECUTOR_EVM: &str = "executor_evm";
pub const EXECUTOR: &str = "executor";
pub const PRE_HASH: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";
pub const DNS4: &str = "dns4";
pub const DEFAULT_BLOCK_INTERVAL: u32 = 3;
pub const DEFAULT_BLOCK_LIMIT: u64 = 100;
pub const CHAIN_CONFIG_FILE: &str = "chain_config.toml";
pub const NODE_CONFIG_FILE: &str = "node_config.toml";
pub const ACCOUNT_DIR: &str = "accounts";
pub const CA_CERT_DIR: &str = "ca_cert";
pub const CERTS_DIR: &str = "certs";
pub const KEY_PEM: &str = "key.pem";
pub const CERT_PEM: &str = "cert.pem";
pub const CSR_PEM: &str = "csr.pem";
pub const LOG4RS_YAML: &str = "log4rs.yaml";
pub const KEY_ID: &str = "key_id";
