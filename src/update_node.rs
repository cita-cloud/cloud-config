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

use crate::config::chain_config::ChainConfig;
use crate::config::consensus_bft::ConsensusBft;
use crate::config::consensus_raft::Consensus as RAFT_Consensus;
use crate::config::controller::ControllerConfig;
use crate::config::executor_evm::ExecutorEvmConfig;
use crate::config::kms_sm::KmsSmConfig;
use crate::config::network_p2p::{NetConfig, PeerConfig as P2P_PeerConfig};
use crate::config::network_tls::{NetworkConfig, PeerConfig as TLS_PeerConfig};
use crate::config::storage_rocksdb::StorageRocksdbConfig;
use crate::constant::{
    CONSENSUS_BFT, CONSENSUS_RAFT, DNS4, EXECUTOR_EVM, KMS_SM, NETWORK_P2P, NETWORK_TLS,
    STORAGE_ROCKSDB,
};
use crate::error::Error;
use crate::traits::TomlWriter;
use crate::traits::YmlWriter;
use crate::util::{read_chain_config, read_file, read_node_config};
use clap::Clap;
use std::fs;

/// A subcommand for run
#[derive(Clap, Debug, Clone)]
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
    /// account of node
    #[clap(long = "account")]
    pub(crate) account: String,
}

/// generate node config files by chain_config and node_config
pub fn execute_update_node(opts: UpdateNodeOpts) -> Result<(), Error> {
    // load chain_config
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, "chain_config.toml"
    );
    let chain_config = read_chain_config(&file_name).unwrap();

    let node_dir = format!("{}/{}-{}", &opts.config_dir, &opts.chain_name, &opts.domain);
    let config_file_name = format!("{}/{}", &node_dir, opts.config_name);

    // delete old config file
    let _ = fs::remove_file(&config_file_name);

    // load node_config
    let file_name = format!("{}/node_config.toml", &node_dir);
    let node_config = read_node_config(file_name).unwrap();

    // move account files info node folder
    let from = format!(
        "{}/{}/accounts/{}/kms.db",
        &opts.config_dir, &opts.chain_name, &opts.account
    );
    let to = format!("{}/kms.db", &node_dir);
    fs::copy(from, to).unwrap();

    // network config file
    // if network_p2p
    if find_micro_service(&chain_config, NETWORK_P2P) {
        let mut uris: Vec<P2P_PeerConfig> = Vec::new();
        for node_network_address in &chain_config.node_network_address_list {
            if node_network_address.domain != opts.domain {
                uris.push(P2P_PeerConfig {
                    address: format!(
                        "/{}/{}/tcp/{}",
                        DNS4, node_network_address.host, node_network_address.port
                    ),
                });
            }
        }
        let network_config = NetConfig::new(
            node_config.network_listen_port,
            node_config.grpc_ports.network_port,
            &uris,
        );
        network_config.write(&config_file_name);
        network_config.write_log4rs(&node_dir);
    } else if find_micro_service(&chain_config, NETWORK_TLS) {
        let mut tls_peers: Vec<TLS_PeerConfig> = Vec::new();
        for node_network_address in &chain_config.node_network_address_list {
            if node_network_address.domain != opts.domain {
                tls_peers.push(crate::config::network_tls::PeerConfig {
                    host: node_network_address.host.clone(),
                    port: node_network_address.port,
                    domain: node_network_address.domain.clone(),
                });
            }
        }
        // load cert
        let ca_cert = read_file(format!(
            "{}/{}/ca_cert/cert.pem",
            &opts.config_dir, &opts.chain_name
        ))
        .unwrap();
        let cert = read_file(format!(
            "{}/{}/certs/{}/cert.pem",
            &opts.config_dir, &opts.chain_name, &opts.domain
        ))
        .unwrap();
        let key = read_file(format!(
            "{}/{}/certs/{}/key.pem",
            &opts.config_dir, &opts.chain_name, &opts.domain
        ))
        .unwrap();

        let network_config = NetworkConfig::new(
            node_config.network_listen_port,
            node_config.grpc_ports.network_port,
            ca_cert,
            cert,
            key,
            tls_peers,
        );
        network_config.write(&config_file_name);
        network_config.write_log4rs(&node_dir);
    } else {
        panic!("unsupport network service");
    }

    // consensus config file
    // if consensus_raft
    if find_micro_service(&chain_config, CONSENSUS_RAFT) {
        let consensus_config = RAFT_Consensus::new_all(
            node_config.grpc_ports.network_port,
            node_config.grpc_ports.controller_port,
            opts.account.clone(),
            node_config.grpc_ports.consensus_port,
        );
        consensus_config.write(&config_file_name);
    } else if find_micro_service(&chain_config, CONSENSUS_BFT) {
        let consensus_config = ConsensusBft::new(
            node_config.grpc_ports.controller_port,
            node_config.grpc_ports.consensus_port,
            node_config.grpc_ports.network_port,
            node_config.grpc_ports.kms_port,
            format!("0x{}", opts.account),
        );
        consensus_config.write(&config_file_name);
        consensus_config.write_log4rs(&node_dir);
    } else {
        panic!("unsupport consensus service");
    }

    // executor config file
    // if executor_evm
    if find_micro_service(&chain_config, EXECUTOR_EVM) {
        let executor_config = ExecutorEvmConfig::new(node_config.grpc_ports.executor_port);
        executor_config.write(&config_file_name);
        executor_config.write_log4rs(&node_dir);
    } else {
        panic!("unsupport executor service");
    }

    // storage config file
    // if storage_rocksdb
    if find_micro_service(&chain_config, STORAGE_ROCKSDB) {
        let storage_config = StorageRocksdbConfig::new(
            node_config.grpc_ports.kms_port,
            node_config.grpc_ports.storage_port,
        );
        storage_config.write(&config_file_name);
        storage_config.write_log4rs(&node_dir);
    } else {
        panic!("unsupport storage service");
    }

    // controller config file
    if find_micro_service(&chain_config, "controller") {
        chain_config.genesis_block.write(&config_file_name);
        chain_config.system_config.write(&config_file_name);
        let controller_config = ControllerConfig {
            network_port: node_config.grpc_ports.network_port,
            consensus_port: node_config.grpc_ports.consensus_port,
            executor_port: node_config.grpc_ports.executor_port,
            storage_port: node_config.grpc_ports.storage_port,
            controller_port: node_config.grpc_ports.controller_port,
            kms_port: node_config.grpc_ports.kms_port,
            key_id: node_config.key_id,
            node_address: opts.account,
            package_limit: node_config.package_limit,
        };
        controller_config.write(&config_file_name);
        controller_config.write_log4rs(&node_dir);
    } else {
        panic!("unsupport controller service");
    }

    // kms config file
    // if kms_sm
    if find_micro_service(&chain_config, KMS_SM) {
        let kms_config = KmsSmConfig::new(node_config.grpc_ports.kms_port, node_config.db_key);
        kms_config.write(&config_file_name);
        kms_config.write_log4rs(&node_dir);
    } else {
        panic!("unsupport kms service");
    }

    Ok(())
}

// pub fn execute_update_network(opts: UpdateNodeOpts) -> Result<(), Error> {
//     // load chain_config
//     let file_name = format!(
//         "{}/{}/{}",
//         &opts.config_dir, &opts.chain_name, "chain_config.toml"
//     );
//     let chain_config = read_chain_config(&file_name).unwrap();
//
//     //delete node folder
//     let path = format!("{}/{}-{}/{}", &opts.config_dir, &opts.chain_name, &opts.domain, &opts.config_name);
//     let config_toml = read_from_file(&path).unwrap();
//     // network config file
//     // if network_p2p
//     if find_micro_service(&chain_config, "network_p2p") {
//         let mut uris: Vec<PeerConfig> = Vec::new();
//         for node_network_address in &chain_config.node_network_address_list {
//             if node_network_address.domain != opts.domain {
//                 uris.push(PeerConfig {
//                     address: format!(
//                         "/dns4/{}/tcp/{}",
//                         node_network_address.host, node_network_address.port
//                     ),
//                 });
//             }
//         }
//         config_toml.network_p2p.unwrap().peers = uris;
//
//
//     } else if find_micro_service(&chain_config, "network_tls") {
//         let mut tls_peers: Vec<TLS_PeerConfig> = Vec::new();
//         for node_network_address in &chain_config.node_network_address_list {
//             if node_network_address.domain != opts.domain {
//                 tls_peers.push(crate::config::network_tls::PeerConfig {
//                     host: node_network_address.host.clone(),
//                     port: node_network_address.port,
//                     domain: node_network_address.domain.clone(),
//                 });
//             }
//         }
//         // load cert
//         let ca_cert = read_file(format!(
//             "{}/{}/ca_cert/cert.pem",
//             &opts.config_dir, &opts.chain_name
//         ))
//             .unwrap();
//         let cert = read_file(format!(
//             "{}/{}/certs/{}/cert.pem",
//             &opts.config_dir, &opts.chain_name, &opts.domain
//         ))
//             .unwrap();
//         let key = read_file(format!(
//             "{}/{}/certs/{}/key.pem",
//             &opts.config_dir, &opts.chain_name, &opts.domain
//         ))
//             .unwrap();
//
//         let network_config = NetworkConfig::new(
//             node_config.network_listen_port,
//             node_config.grpc_ports.network_port,
//             ca_cert,
//             cert,
//             key,
//             tls_peers,
//         );
//         network_config.write(&config_file_name);
//         network_config.write_log4rs(&node_dir);
//     } else {
//         panic!("unsupport network service");
//     }
//
//     OK(())
// }

pub fn find_micro_service(chain_config: &ChainConfig, service_name: &str) -> bool {
    for micro_service in &chain_config.micro_service_list {
        if micro_service.image == service_name {
            return true;
        }
    }
    false
}
