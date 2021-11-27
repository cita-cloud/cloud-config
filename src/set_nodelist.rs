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

use crate::config::chain_config::{NodeNetworkAddress, NodeNetworkAddressBuilder};
use crate::error::Error;
use crate::util::{read_chain_config, write_toml};
use clap::Clap;

/// A subcommand for run
#[derive(Clap, Debug, Clone)]
pub struct SetNodeListOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub(crate) chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub(crate) config_dir: String,
    /// node list looks like localhost:40000:node0,localhost:40001:node1
    #[clap(long = "nodelist")]
    pub(crate) node_list: String,
}

/// execute set node list
pub fn execute_set_nodelist(opts: SetNodeListOpts) -> Result<(), Error> {
    // load chain_config
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, "chain_config.toml"
    );
    let mut chain_config = read_chain_config(&file_name).unwrap();

    let node_list_str: Vec<&str> = opts.node_list.split(',').collect();
    let node_list = node_list_str
        .iter()
        .map(|node| {
            let node_network_info: Vec<&str> = node.split(':').collect();
            NodeNetworkAddressBuilder::new()
                .host(node_network_info[0].to_string())
                .port(node_network_info[1].parse::<u16>().unwrap())
                .domain(node_network_info[2].to_string())
                .build()
        })
        .collect();

    chain_config.set_node_network_address_list(node_list);

    // store chain_config
    write_toml(&chain_config, file_name);

    Ok(())
}

pub fn get_old_node_list_count(opts: SetNodeListOpts) -> Vec<NodeNetworkAddress> {
    // load chain_config
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, "chain_config.toml"
    );
    let chain_config = read_chain_config(&file_name).unwrap();
    chain_config.node_network_address_list
}