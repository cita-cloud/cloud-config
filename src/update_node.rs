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

use crate::config::consensus_overlord::ConsensusOverlord;
use crate::config::consensus_raft::Consensus as RAFT_Consensus;
use crate::config::controller::ControllerConfig;
use crate::config::executor_evm::ExecutorEvmConfig;
use crate::config::log_config::LogConfig;
use crate::config::network_zenoh::{ModuleConfig, PeerConfig as ZenohPeerConfig, ZenohConfig};
use crate::config::storage_opendal::StorageOpendalConfig;
use crate::constant::{
    ACCOUNT_DIR, CA_CERT_DIR, CERTS_DIR, CERT_PEM, CHAIN_CONFIG_FILE, CONSENSUS,
    CONSENSUS_OVERLORD, CONSENSUS_RAFT, CONTROLLER, CONTROLLER_HSM, EXECUTOR, EXECUTOR_EVM,
    KEY_PEM, NETWORK, NETWORK_ZENOH, NODE_ADDRESS, NODE_CONFIG_FILE, PRIVATE_KEY, STORAGE,
    STORAGE_OPENDAL, VALIDATOR_ADDRESS,
};
use crate::error::Error;
use crate::traits::TomlWriter;
use crate::util::{find_micro_service, read_chain_config, read_file, read_node_config};
use clap::Parser;
use std::fs;
use std::net::Ipv4Addr;

/// A subcommand for run
#[derive(Parser, Debug, Clone)]
pub struct UpdateNodeOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub config_dir: String,
    /// set config file name
    #[clap(long = "config-name", default_value = "config.toml")]
    pub config_name: String,
    /// domain of node
    #[clap(long = "domain")]
    pub domain: String,
}

/// generate node config files by chain_config and node_config
pub fn execute_update_node(opts: UpdateNodeOpts) -> Result<(), Error> {
    let node_dir = format!("{}/{}-{}", &opts.config_dir, &opts.chain_name, &opts.domain);

    // load node_config
    let file_name = format!("{}/{}", &node_dir, NODE_CONFIG_FILE);
    let node_config = read_node_config(file_name).unwrap();

    // load chain_config
    let file_name = format!("{}/{}", &node_dir, CHAIN_CONFIG_FILE);
    let chain_config = read_chain_config(file_name).unwrap();

    let mut my_cluster_name = "";
    let mut my_external_port = 0;
    let mut my_name_space = "";
    for node_network_address in &chain_config.node_network_address_list {
        if node_network_address.domain == opts.domain {
            my_cluster_name = &node_network_address.cluster;
            my_external_port = node_network_address.port;
            my_name_space = &node_network_address.name_space;
        }
    }

    if my_external_port == 0 {
        panic!("can't find domain in chain_config.node_network_address_list");
    }

    let is_k8s = !my_cluster_name.is_empty();

    // because this file write by one and one section
    // so write mode must be append
    // so if you want rewrite, delete old config file at first
    let config_file_name = format!("{}/{}", &node_dir, opts.config_name);
    let _ = fs::remove_file(&config_file_name);

    // copy account files
    {
        let from = format!(
            "{}/{}/{}/{}",
            &node_dir, ACCOUNT_DIR, &node_config.account, PRIVATE_KEY
        );
        let to = format!("{}/{}", &node_dir, PRIVATE_KEY);
        fs::copy(from, to).unwrap();

        let from = format!(
            "{}/{}/{}/{}",
            &node_dir, ACCOUNT_DIR, &node_config.account, VALIDATOR_ADDRESS
        );
        let to = format!("{}/{}", &node_dir, VALIDATOR_ADDRESS);
        fs::copy(from, to).unwrap();

        let from = format!(
            "{}/{}/{}/{}",
            &node_dir, ACCOUNT_DIR, &node_config.account, NODE_ADDRESS
        );
        let to = format!("{}/{}", &node_dir, NODE_ADDRESS);
        fs::copy(from, to).unwrap();
    }

    let real_domain = format!("{}-{}", &opts.chain_name, &opts.domain);

    // network config file
    // config peers
    // if current node in k8s
    // -- if same cluster and same namespace port is 40000, domain is svc name
    // -- if same cluster and different namespace, port is 40000, domain is svc name
    // -- if diffrent cluster and peer host is FQDN, port is peer port, domain is svc name
    // -- if diffrent cluster and peer host is ip, port is 40000, domain is svc name
    // if current node not in k8s, port is peer port, domain is peer host
    if find_micro_service(&chain_config, NETWORK_ZENOH) {
        let mut zenoh_peers: Vec<ZenohPeerConfig> = Vec::new();
        for node_network_address in &chain_config.node_network_address_list {
            if node_network_address.domain != opts.domain {
                if is_k8s {
                    let peer_cluster_name = &node_network_address.cluster;
                    let peer_name_space = &node_network_address.name_space;
                    let peer_host = &node_network_address.host;
                    let is_peer_host_ip = peer_host.parse::<Ipv4Addr>().is_ok();
                    let peer_port = node_network_address.port;
                    let peer_svc_name =
                        format!("{}-{}", &opts.chain_name, &node_network_address.domain);

                    let same_cluster = peer_cluster_name == my_cluster_name;
                    let same_name_space = peer_name_space == my_name_space;

                    let peer_config = match (same_cluster, same_name_space, is_peer_host_ip) {
                        (true, true, _) => ZenohPeerConfig {
                            port: 40000,
                            domain: peer_svc_name,
                            protocol: "quic".to_string(),
                        },
                        (true, false, _) => ZenohPeerConfig {
                            port: 40000,
                            domain: peer_svc_name,
                            protocol: "quic".to_string(),
                        },
                        (false, _, false) => ZenohPeerConfig {
                            port: peer_port,
                            domain: peer_svc_name,
                            protocol: "quic".to_string(),
                        },
                        (false, _, true) => ZenohPeerConfig {
                            port: 40000,
                            domain: peer_svc_name,
                            protocol: "quic".to_string(),
                        },
                    };

                    zenoh_peers.push(peer_config);
                } else {
                    zenoh_peers.push(ZenohPeerConfig {
                        port: node_network_address.port,
                        domain: node_network_address.host.clone(),
                        protocol: "quic".to_string(),
                    });
                }
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
            port: if is_k8s { 40000 } else { my_external_port },
            grpc_port: node_config.grpc_ports.network_port,
            ca_cert,
            cert,
            priv_key: key,
            peers: zenoh_peers,
            domain: real_domain.clone(),
            protocol: "quic".to_string(),
            node_address: if is_k8s {
                format!("/mnt/{NODE_ADDRESS}")
            } else {
                NODE_ADDRESS.to_string()
            },
            validator_address: if is_k8s {
                format!("/mnt/{VALIDATOR_ADDRESS}")
            } else {
                VALIDATOR_ADDRESS.to_string()
            },
            chain_id: chain_config.system_config.chain_id.clone(),
            modules,
            metrics_port: node_config.metrics_ports.network_metrics_port,
            enable_metrics: node_config.enable_metrics,
            log_config: LogConfig {
                max_level: node_config.log_level.clone(),
                service_name: NETWORK.to_owned(),
                rolling_file_path: node_config.log_file_path.clone(),
                agent_endpoint: node_config.jaeger_agent_endpoint.clone(),
                filter: node_config.log_level.clone(),
            },
        };
        network_config.write(&config_file_name);
    } else {
        panic!("unsupport network service");
    }

    // consensus config file
    // if consensus_raft
    if find_micro_service(&chain_config, CONSENSUS_RAFT) {
        let consensus_config = RAFT_Consensus::new(
            node_config.grpc_ports.network_port,
            node_config.grpc_ports.controller_port,
            if is_k8s {
                format!("/mnt/{NODE_ADDRESS}")
            } else {
                NODE_ADDRESS.to_string()
            },
            node_config.grpc_ports.consensus_port,
            node_config.metrics_ports.consensus_metrics_port,
            node_config.enable_metrics,
        );
        consensus_config.write(&config_file_name);
    } else if find_micro_service(&chain_config, CONSENSUS_OVERLORD) {
        let consensus_config = ConsensusOverlord::new(
            real_domain.clone(),
            node_config.grpc_ports.controller_port,
            node_config.grpc_ports.consensus_port,
            node_config.grpc_ports.network_port,
            node_config.metrics_ports.consensus_metrics_port,
            node_config.enable_metrics,
            LogConfig {
                max_level: node_config.log_level.clone(),
                service_name: CONSENSUS.to_owned(),
                rolling_file_path: node_config.log_file_path.clone(),
                agent_endpoint: node_config.jaeger_agent_endpoint.clone(),
                filter: node_config.log_level.clone(),
            },
        );
        consensus_config.write(&config_file_name);
    } else {
        panic!("unsupport consensus service");
    }

    // executor config file
    // if executor_evm
    if find_micro_service(&chain_config, EXECUTOR_EVM) {
        let executor_config = ExecutorEvmConfig::new(
            real_domain.clone(),
            node_config.grpc_ports.executor_port,
            node_config.metrics_ports.executor_metrics_port,
            node_config.enable_metrics,
            LogConfig {
                max_level: node_config.log_level.clone(),
                service_name: EXECUTOR.to_owned(),
                rolling_file_path: node_config.log_file_path.clone(),
                agent_endpoint: node_config.jaeger_agent_endpoint.clone(),
                filter: node_config.log_level.clone(),
            },
        );
        executor_config.write(&config_file_name);
    } else {
        panic!("unsupport executor service");
    }

    // storage config file
    // if storage_opendal
    if find_micro_service(&chain_config, STORAGE_OPENDAL) {
        let storage_config = StorageOpendalConfig::new(
            real_domain.clone(),
            node_config.grpc_ports.storage_port,
            node_config.metrics_ports.storage_metrics_port,
            node_config.enable_metrics,
            LogConfig {
                max_level: node_config.log_level.clone(),
                service_name: STORAGE.to_owned(),
                rolling_file_path: node_config.log_file_path.clone(),
                agent_endpoint: node_config.jaeger_agent_endpoint.clone(),
                filter: node_config.log_level.clone(),
            },
            node_config.cloud_storage.clone(),
        );
        storage_config.write(&config_file_name);
    } else {
        panic!("unsupport storage service");
    }

    // controller config file
    if find_micro_service(&chain_config, CONTROLLER)
        || find_micro_service(&chain_config, CONTROLLER_HSM)
    {
        chain_config.genesis_block.write(&config_file_name);
        chain_config.system_config.write(&config_file_name);
        let controller_config = ControllerConfig {
            domain: real_domain,
            network_port: node_config.grpc_ports.network_port,
            consensus_port: node_config.grpc_ports.consensus_port,
            executor_port: node_config.grpc_ports.executor_port,
            storage_port: node_config.grpc_ports.storage_port,
            controller_port: node_config.grpc_ports.controller_port,
            node_address: if is_k8s {
                format!("/mnt/{NODE_ADDRESS}")
            } else {
                NODE_ADDRESS.to_string()
            },
            validator_address: if is_k8s {
                format!("/mnt/{VALIDATOR_ADDRESS}")
            } else {
                VALIDATOR_ADDRESS.to_string()
            },
            metrics_port: node_config.metrics_ports.controller_metrics_port,
            enable_metrics: node_config.enable_metrics,
            log_config: LogConfig {
                max_level: node_config.log_level.clone(),
                service_name: CONTROLLER.to_owned(),
                rolling_file_path: node_config.log_file_path.clone(),
                agent_endpoint: node_config.jaeger_agent_endpoint.clone(),
                filter: node_config.log_level.clone(),
            },
            is_danger: node_config.is_danger,
            tx_persistence: node_config.enable_tx_persistence,
        };
        controller_config.write(&config_file_name);
    } else {
        panic!("unsupport controller service");
    }

    Ok(())
}
