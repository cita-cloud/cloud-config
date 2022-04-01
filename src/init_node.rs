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

use crate::config::chain_config::ConfigStage;
use crate::config::node_config::{GrpcPortsBuilder, NodeConfigBuilder};
use crate::constant::{CHAIN_CONFIG_FILE, NODE_CONFIG_FILE};
use crate::error::Error;
use crate::util::{copy_dir_all, read_chain_config, write_toml};
use clap::Parser;
use std::path::Path;

/// A subcommand for run
#[derive(Parser, Debug, Clone)]
pub struct InitNodeOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub(crate) chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub(crate) config_dir: String,
    /// domain of node
    #[clap(long = "domain")]
    pub(crate) domain: String,
    /// grpc network_port of node
    #[clap(long = "network-port", default_value = "50000")]
    pub(crate) network_port: u16,
    /// grpc consensus_port of node
    #[clap(long = "consensus-port", default_value = "50001")]
    pub(crate) consensus_port: u16,
    /// grpc executor_port of node
    #[clap(long = "executor-port", default_value = "50002")]
    pub(crate) executor_port: u16,
    /// grpc storage_port of node
    #[clap(long = "storage-port", default_value = "50003")]
    pub(crate) storage_port: u16,
    /// grpc controller_port of node
    #[clap(long = "controller-port", default_value = "50004")]
    pub(crate) controller_port: u16,
    /// grpc kms_port of node
    #[clap(long = "kms-port", default_value = "50005")]
    pub(crate) kms_port: u16,
    /// network listen port of node
    #[clap(long = "network-listen-port", default_value = "40000")]
    pub(crate) network_listen_port: u16,
    /// kms db password
    #[clap(long = "kms-password", default_value = "123456")]
    pub(crate) kms_password: String,
    /// key id of account in kms db
    #[clap(long = "key-id", default_value = "1")]
    pub(crate) key_id: u64,
    /// set one block contains tx limit, default 30000
    #[clap(long = "package-limit", default_value = "30000")]
    pub(crate) package_limit: u64,
    /// log level
    #[clap(long = "log-level", default_value = "info")]
    pub(crate) log_level: String,
    /// account of node
    #[clap(long = "account")]
    pub(crate) account: String,
}

/// execute set validators
pub fn execute_init_node(opts: InitNodeOpts) -> Result<(), Error> {
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CHAIN_CONFIG_FILE
    );

    if Path::new(&file_name).exists() {
        let chain_config = read_chain_config(&file_name).unwrap();
        // gen node config after chain config stage is Finalize
        if chain_config.stage != ConfigStage::Finalize {
            return Err(Error::InvalidStage);
        }
    } else {
        return Err(Error::InvalidStage);
    }

    let grpc_ports = GrpcPortsBuilder::new()
        .network_port(opts.network_port)
        .consensus_port(opts.consensus_port)
        .executor_port(opts.executor_port)
        .storage_port(opts.storage_port)
        .controller_port(opts.controller_port)
        .kms_port(opts.kms_port)
        .build();
    let node_config = NodeConfigBuilder::new()
        .grpc_ports(grpc_ports)
        .network_listen_port(opts.network_listen_port)
        .db_key(opts.kms_password)
        .key_id(opts.key_id)
        .package_limit(opts.package_limit)
        .log_level(opts.log_level)
        .account(opts.account)
        .build();

    let node_dir = format!("{}/{}-{}", &opts.config_dir, &opts.chain_name, &opts.domain);
    let from = format!("{}/{}", &opts.config_dir, &opts.chain_name);
    copy_dir_all(&from, &node_dir).unwrap();

    let file_name = format!("{}/{}", &node_dir, NODE_CONFIG_FILE);
    write_toml(&node_config, file_name);

    Ok(())
}
