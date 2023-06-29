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
use crate::config::chain_config::{ChainConfigBuilder, MicroService, MicroServiceBuilder};
use crate::config::controller::{GenesisBlockBuilder, SystemConfigBuilder};
use crate::constant::CHAIN_CONFIG_FILE;
use crate::error::Error;
use crate::util::{read_chain_config, sm3_hash, unix_now, write_toml};
use clap::Parser;
use std::path::Path;

/// A subcommand for run
#[derive(Parser, Debug, Clone)]
pub struct InitChainConfigOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub(crate) chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub(crate) config_dir: String,
    /// set genesis timestamp
    #[clap(long = "timestamp", default_value = "0")]
    pub(crate) timestamp: u64,
    /// set genesis prevhash
    #[clap(
        long = "prevhash",
        default_value = "0x0000000000000000000000000000000000000000000000000000000000000000"
    )]
    pub(crate) prevhash: String,
    /// set system config version
    #[clap(long = "version", default_value = "0")]
    pub(crate) version: u32,
    /// set system config chain_id
    #[clap(long = "chain_id", default_value = "")]
    pub(crate) chain_id: String,
    /// set system config block_interval
    #[clap(long = "block_interval", default_value = "3")]
    pub(crate) block_interval: u32,
    /// set system config block_limit
    #[clap(long = "block_limit", default_value = "100")]
    pub(crate) block_limit: u64,
    /// set one block contains quota limit, default 1073741824
    #[clap(long = "quota-limit", default_value = "1073741824")]
    pub(crate) quota_limit: u64,
    /// set network micro service image name (network_zenoh)
    #[clap(long = "network_image", default_value = "network_zenoh")]
    pub(crate) network_image: String,
    /// set network micro service image tag
    #[clap(long = "network_tag", default_value = "latest")]
    pub(crate) network_tag: String,
    /// set consensus micro service image name (consensus_raft/consensus_overlord)
    #[clap(long = "consensus_image", default_value = "consensus_overlord")]
    pub(crate) consensus_image: String,
    /// set consensus micro service image tag
    #[clap(long = "consensus_tag", default_value = "latest")]
    pub(crate) consensus_tag: String,
    /// set executor micro service image name (executor_evm)
    #[clap(long = "executor_image", default_value = "executor_evm")]
    pub(crate) executor_image: String,
    /// set executor micro service image tag
    #[clap(long = "executor_tag", default_value = "latest")]
    pub(crate) executor_tag: String,
    /// set storage micro service image name (storage_opendal)
    #[clap(long = "storage_image", default_value = "storage_opendal")]
    pub(crate) storage_image: String,
    /// set storage micro service image tag
    #[clap(long = "storage_tag", default_value = "latest")]
    pub(crate) storage_tag: String,
    /// set controller micro service image name (controller)
    #[clap(long = "controller_image", default_value = "controller")]
    pub(crate) controller_image: String,
    /// set controller micro service image tag
    #[clap(long = "controller_tag", default_value = "latest")]
    pub(crate) controller_tag: String,
}

/// init chain config
/// $(config_dir)
/// --  $(chain_name)
/// ------  chain_config.toml
pub fn execute_init_chain_config(opts: InitChainConfigOpts) -> Result<(), Error> {
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CHAIN_CONFIG_FILE
    );

    if Path::new(&file_name).exists() {
        let chain_config = read_chain_config(&file_name).unwrap();
        // rewrite chain config only when stage is Init
        if chain_config.stage != ConfigStage::Init {
            return Err(Error::InvalidStage);
        }
    }

    // pre proc timestamp and chain_id
    let timestamp = if opts.timestamp == 0 {
        unix_now()
    } else {
        opts.timestamp
    };

    let chain_id = if opts.chain_id.is_empty() {
        hex::encode(sm3_hash(opts.chain_name.as_bytes()))
    } else {
        opts.chain_id
    };

    // proc five micro service
    let network_micro_service = MicroServiceBuilder::default()
        .image(opts.network_image)
        .tag(opts.network_tag)
        .build();
    let consensus_micro_service = MicroServiceBuilder::default()
        .image(opts.consensus_image)
        .tag(opts.consensus_tag)
        .build();
    let executor_micro_service = MicroServiceBuilder::default()
        .image(opts.executor_image)
        .tag(opts.executor_tag)
        .build();
    let storage_micro_service = MicroServiceBuilder::default()
        .image(opts.storage_image)
        .tag(opts.storage_tag)
        .build();
    let controller_micro_service = MicroServiceBuilder::default()
        .image(opts.controller_image)
        .tag(opts.controller_tag)
        .build();

    let micro_service_list: Vec<MicroService> = vec![
        network_micro_service,
        consensus_micro_service,
        executor_micro_service,
        storage_micro_service,
        controller_micro_service,
    ];

    // genesis block
    let genesis_block = GenesisBlockBuilder::default()
        .timestamp(timestamp)
        .prevhash(opts.prevhash)
        .build();

    // system config
    let system_config = SystemConfigBuilder::default()
        .version(opts.version)
        .chain_id(chain_id)
        .block_interval(opts.block_interval)
        .block_limit(opts.block_limit)
        .quota_limit(opts.quota_limit)
        .build();

    let chain_config = ChainConfigBuilder::default()
        .system_config(system_config)
        .genesis_block(genesis_block)
        .micro_service_list(micro_service_list)
        .build();

    write_toml(chain_config, file_name);

    Ok(())
}
