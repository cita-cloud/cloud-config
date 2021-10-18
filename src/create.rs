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

use std::fs;
use clap::Clap;
use crate::traits::Kms;
use crate::error::{Error, Result};
use crate::config::admin::AdminConfig;
use crate::config::controller::ControllerConfig;

/// A subcommand for run
#[derive(Clap, Debug)]
pub struct CreateOpts {
    /// set config file name
    #[clap(long = "config-name", default_value = "config.toml")]
    config_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir")]
    config_dir: Option<String>,
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    chain_name: String,
    /// Set controller micro-service.
    #[clap(long = "controller", default_value = "controller")]
    controller: String,
    /// Set consensus micro-service.
    #[clap(long = "consensus")]
    consensus: String,
    /// Set executor micro-service.
    #[clap(long = "executor", default_value = "executor_evm")]
    executor: String,
    /// Set network micro-service.
    #[clap(long = "network")]
    network: String,
    /// Set kms micro-service.
    #[clap(long = "kms", default_value = "kms_sm")]
    kms: String,
    /// Set storage micro-service.
    #[clap(long = "storage", default_value = "storage_rocksdb")]
    storage: String,
    /// grpc port list, input "p1,p2,p3,p4", use default grpc port count from 50000 + 1000 * i
    /// use default must set peer_count or p2p_ports
    #[clap(long = "grpc-ports", default_value = "default")]
    grpc_ports: String,
    /// p2p port list, input "ip1:port1,ip2:port2,ip3:port3,ip4:port4", use default port count from
    /// 127.0.0.1:40000 + 1 * i, use default must set peer_count or grpc_ports
    #[clap(long = "p2p-ports", default_value = "default")]
    p2p_ports: String,
    /// set initial node number, default "none" mean not use this must set grpc_ports or p2p_ports,
    /// if set peers_count, grpc_ports and p2p_ports, base on grpc_ports > p2p_ports > peers_count
    #[clap(long = "peers-count")]
    peers_count: Option<u16>,
    /// kms db password
    #[clap(long = "kms-password", default_value = "123456")]
    kms_password: String,
    /// set one block contains tx limit, default 30000
    #[clap(long = "package-limit", default_value = "30000")]
    package_limit: u64,
}

impl CreateOpts {
    fn admin_dir(&self) -> String {
        if let Some(dir) = &self.config_dir {
            format!("{}/{}", dir, &self.chain_name)
        } else {
            format!("{}", &self.chain_name)
        }
    }

    fn get_dir(&self, index: u16) -> String {
        if let Some(dir) = &self.config_dir {
            format!("{}/{}-{}", dir, &self.chain_name, index)
        } else {
            format!("{}-{}", &self.chain_name, index)
        }
    }
}

pub fn execute_create(opts: CreateOpts) -> Result {
    if let Some(dir) = &opts.config_dir {
        fs::create_dir_all(dir).unwrap();
    }

    // admin account dir
    let admin_dir = opts.admin_dir();

    if opts.grpc_ports == "default" {
        if opts.peers_count == "default" {
            if let Some(num) = opts.peers_count {
                // network template


                // kms
                let kms = match opts.kms.as_str() {
                    "kms_sm" => crate::config::kms_sm::Kms::create_kms_db(admin_dir.clone(), opts.kms_password.clone()),
                    other => {
                        log::warn!("kms server chose error, input: {}", other);
                        Err(Error::KmsNotDefaultOrKmsSm)
                    }
                };
                let (key_id, address) = kms.generate_key_pair("create by cmd".to_string());
                let admin_config = AdminConfig {
                    db_key: opts.kms_password.clone(),
                    key_id,
                    db_path: "kms.db".to_string(),
                    admin_address: format!("0x{}", hex::encode(address)),
                };
                admin_config.write_to_file(format!("{}/config.toml", &admin_dir));
                for i in 0..num {
                    let network_grpc_port = 50000 + i;
                    let network_p2p_port = 40000 + i;

                    let node_dir = opts.get_dir(i);
                    let kms = match opts.kms.as_str() {
                        "kms_sm" => crate::config::kms_sm::Kms::create_kms_db(node_dir.clone(), opts.kms_password.clone()),
                        other => {
                            log::warn!("kms server chose error, input: {}", other);
                            Err(Error::KmsNotDefaultOrKmsSm)
                        }
                    };
                    let (key_id, address) = kms.generate_key_pair("create by cmd".to_string());
                    let controller_config = ControllerConfig {
                        network_port: network_grpc_port,
                        consensus_port: network_grpc_port + 1,
                        executor_port: network_grpc_port + 2,
                        storage_port: network_grpc_port + 3,
                        controller_port: network_grpc_port + 4,
                        kms_port: network_grpc_port + 5,
                        key_id,
                        node_address: format!("0x{}", hex::encode(address)),
                        package_limit: 30000
                    };
                    controller_config.write_to_file(format!("{}/config.toml", &node_dir));
                }
            } else {
                log::warn!("must set one of grpc_ports, peers_count or peers_count");
            }
        }
    }

    Ok(())
}
