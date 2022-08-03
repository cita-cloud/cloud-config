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

use crate::config::consensus_bft::ConsensusBft;
use crate::config::consensus_overlord::ConsensusOverlord;
use crate::config::consensus_raft::Consensus as RAFT_Consensus;
use crate::config::controller::ControllerConfig;
use crate::config::crypto_eth::CryptoEthConfig;
use crate::config::crypto_sm::CryptoSmConfig;
use crate::config::executor_evm::ExecutorEvmConfig;
use crate::config::network_zenoh::{ModuleConfig, PeerConfig as ZenohPeerConfig, ZenohConfig};
use crate::config::storage_rocksdb::StorageRocksdbConfig;
use crate::constant::{
    ACCOUNT_DIR, CA_CERT_DIR, CERTS_DIR, CERT_PEM, CHAIN_CONFIG_FILE, CONSENSUS_BFT,
    CONSENSUS_OVERLORD, CONSENSUS_RAFT, CONTROLLER, CRYPTO_ETH, CRYPTO_SM, EXECUTOR_EVM, KEY_PEM,
    NETWORK_ZENOH, NODE_CONFIG_FILE, PRIVATE_KEY, STORAGE_ROCKSDB, VALIDATOR_ADDRESS,
};
use crate::error::Error;
use crate::traits::TomlWriter;
use crate::traits::YmlWriter;
use crate::util::{find_micro_service, read_chain_config, read_file, read_node_config};
use clap::Parser;
use std::fs;

/// A subcommand for run
#[derive(Parser, Debug, Clone)]
pub struct UpdateNodeOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub(crate) chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub(crate) config_dir: String,
    /// set config file name
    #[clap(long = "config-name", default_value = "config.toml")]
    pub(crate) config_name: String,
    /// domain of node
    #[clap(long = "domain")]
    pub(crate) domain: String,
    /// disable output to stdout
    #[clap(long = "no-stdout")]
    pub(crate) no_stdout: bool,
    /// is old node
    #[clap(long = "is-old")]
    pub(crate) is_old: bool,
}

/// generate node config files by chain_config and node_config
pub fn execute_update_node(opts: UpdateNodeOpts) -> Result<(), Error> {
    let is_old = opts.is_old;
    let is_stdout = !opts.no_stdout;

    let node_dir = format!("{}/{}-{}", &opts.config_dir, &opts.chain_name, &opts.domain);

    // load node_config
    let file_name = format!("{}/{}", &node_dir, NODE_CONFIG_FILE);
    let node_config = read_node_config(file_name).unwrap();

    // load chain_config
    let file_name = format!("{}/{}", &node_dir, CHAIN_CONFIG_FILE);
    let chain_config = read_chain_config(&file_name).unwrap();

    let mut local_cluster = "";
    for node_network_address in &chain_config.node_network_address_list {
        if node_network_address.domain == opts.domain {
            local_cluster = &node_network_address.cluster;
        }
    }

    // because this file write by one and one section
    // so write mode must be append
    // so if you want rewrite, delete old config file at first
    let config_file_name = format!("{}/{}", &node_dir, opts.config_name);
    let _ = fs::remove_file(&config_file_name);

    let is_overlord = find_micro_service(&chain_config, CONSENSUS_OVERLORD);

    // copy account files
    // only for new node
    if !is_old {
        let from = format!(
            "{}/{}/{}/{}",
            &node_dir, ACCOUNT_DIR, &node_config.account, PRIVATE_KEY
        );
        let to = format!("{}/{}", &node_dir, PRIVATE_KEY);
        fs::copy(from, to).unwrap();
    }

    // network config file
    if find_micro_service(&chain_config, NETWORK_ZENOH) {
        let mut zenoh_peers: Vec<ZenohPeerConfig> = Vec::new();
        for node_network_address in &chain_config.node_network_address_list {
            if node_network_address.domain != opts.domain {
                let real_domain = format!("{}-{}", &opts.chain_name, &node_network_address.domain);
                let node_cluster = &node_network_address.cluster;
                let port = if local_cluster == node_cluster {
                    node_network_address.svc_port
                } else {
                    node_network_address.port
                };
                zenoh_peers.push(ZenohPeerConfig {
                    port,
                    domain: real_domain,
                    protocol: "quic".to_string(),
                });
            }
        }
        // load cert
        let ca_cert = read_file(format!("{}/{}/{}", &node_dir, CA_CERT_DIR, CERT_PEM)).unwrap();
        let cert = read_file(format!(
            "{}/{}/{}/{}",
            &node_dir, CERTS_DIR, &opts.domain, CERT_PEM
        ))
        .unwrap();
        let key = read_file(format!(
            "{}/{}/{}/{}",
            &node_dir, CERTS_DIR, &opts.domain, KEY_PEM
        ))
        .unwrap();

        let validator_address_path = format!(
            "{}/{}/{}/{}",
            &node_dir, ACCOUNT_DIR, &node_config.account, VALIDATOR_ADDRESS
        );
        let validator_address = read_file(&validator_address_path).unwrap();
        let real_domain = format!("{}-{}", &opts.chain_name, &opts.domain);

        // modules
        let modules = vec![
            ModuleConfig {
                module_name: "consensus".to_string(),
                hostname: "127.0.0.1".to_string(),
                port: node_config.grpc_ports.consensus_port,
            },
            ModuleConfig {
                module_name: "controller".to_string(),
                hostname: "127.0.0.1".to_string(),
                port: node_config.grpc_ports.controller_port,
            },
        ];

        let network_config = ZenohConfig {
            port: node_config.network_listen_port,
            grpc_port: node_config.grpc_ports.network_port,
            ca_cert,
            cert,
            priv_key: key,
            peers: zenoh_peers,
            domain: real_domain,
            protocol: "quic".to_string(),
            node_address: node_config.account.clone(),
            validator_address,
            chain_id: chain_config.system_config.chain_id.clone(),
            modules,
        };
        network_config.write(&config_file_name);
        // only for new node
        if !is_old {
            network_config.write_log4rs(&node_dir, is_stdout, &node_config.log_level);
        }
    } else {
        panic!("unsupport network service");
    }

    // consensus config file
    // if consensus_raft
    if find_micro_service(&chain_config, CONSENSUS_RAFT) {
        let consensus_config = RAFT_Consensus::new(
            node_config.grpc_ports.network_port,
            node_config.grpc_ports.controller_port,
            node_config.account.clone(),
            node_config.grpc_ports.consensus_port,
        );
        consensus_config.write(&config_file_name);
        // only for new node
        if !is_old {
            consensus_config.write_log4rs(&node_dir, is_stdout, &node_config.log_level);
        }
    } else if find_micro_service(&chain_config, CONSENSUS_BFT) {
        let consensus_config = ConsensusBft::new(
            node_config.grpc_ports.controller_port,
            node_config.grpc_ports.consensus_port,
            node_config.grpc_ports.network_port,
            node_config.grpc_ports.crypto_port,
            format!("0x{}", &node_config.account),
        );
        consensus_config.write(&config_file_name);
        // only for new node
        if !is_old {
            consensus_config.write_log4rs(&node_dir, is_stdout, &node_config.log_level);
        }
    } else if find_micro_service(&chain_config, CONSENSUS_OVERLORD) {
        let validator_address_path = format!(
            "{}/{}/{}/{}",
            &node_dir, ACCOUNT_DIR, &node_config.account, VALIDATOR_ADDRESS
        );
        let validator_address = read_file(&validator_address_path).unwrap();
        let consensus_config = ConsensusOverlord::new(
            node_config.grpc_ports.controller_port,
            node_config.grpc_ports.consensus_port,
            node_config.grpc_ports.network_port,
            validator_address,
        );
        consensus_config.write(&config_file_name);
        // only for new node
        if !is_old {
            consensus_config.write_log4rs(&node_dir, is_stdout, &node_config.log_level);
        }
    } else {
        panic!("unsupport consensus service");
    }

    // executor config file
    // if executor_evm
    if find_micro_service(&chain_config, EXECUTOR_EVM) {
        let executor_config = ExecutorEvmConfig::new(node_config.grpc_ports.executor_port);
        executor_config.write(&config_file_name);
        // only for new node
        if !is_old {
            executor_config.write_log4rs(&node_dir, is_stdout, &node_config.log_level);
        }
    } else {
        panic!("unsupport executor service");
    }

    // storage config file
    // if storage_rocksdb
    if find_micro_service(&chain_config, STORAGE_ROCKSDB) {
        let storage_config = StorageRocksdbConfig::new(
            node_config.grpc_ports.crypto_port,
            node_config.grpc_ports.storage_port,
        );
        storage_config.write(&config_file_name);
        // only for new node
        if !is_old {
            storage_config.write_log4rs(&node_dir, is_stdout, &node_config.log_level);
        }
    } else {
        panic!("unsupport storage service");
    }

    // controller config file
    if find_micro_service(&chain_config, CONTROLLER) {
        chain_config.genesis_block.write(&config_file_name);
        chain_config.system_config.write(&config_file_name);
        let controller_config = ControllerConfig {
            network_port: node_config.grpc_ports.network_port,
            consensus_port: node_config.grpc_ports.consensus_port,
            executor_port: node_config.grpc_ports.executor_port,
            storage_port: node_config.grpc_ports.storage_port,
            controller_port: node_config.grpc_ports.controller_port,
            crypto_port: node_config.grpc_ports.crypto_port,
            node_address: node_config.account.clone(),
            quota_limit: node_config.quota_limit,
            validator_address_len: if is_overlord { 48 } else { 20 },
        };
        controller_config.write(&config_file_name);
        // only for new node
        if !is_old {
            controller_config.write_log4rs(&node_dir, is_stdout, &node_config.log_level);
        }
    } else {
        panic!("unsupport controller service");
    }

    // crypto config file
    // if crypto_sm
    if find_micro_service(&chain_config, CRYPTO_SM) {
        let crypto_config = CryptoSmConfig::new(node_config.grpc_ports.crypto_port);
        crypto_config.write(&config_file_name);
        // only for new node
        if !is_old {
            crypto_config.write_log4rs(&node_dir, is_stdout, &node_config.log_level);
        }
    } else if find_micro_service(&chain_config, CRYPTO_ETH) {
        let crypto_config = CryptoEthConfig::new(node_config.grpc_ports.crypto_port);
        crypto_config.write(&config_file_name);
        // only for new node
        if !is_old {
            crypto_config.write_log4rs(&node_dir, is_stdout, &node_config.log_level);
        }
    } else {
        panic!("unsupport crypto service");
    }

    Ok(())
}
