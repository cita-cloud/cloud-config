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
use crate::config::chain_config::NodeNetworkAddressBuilder;
use crate::constant::CHAIN_CONFIG_FILE;
use crate::error::Error;
use crate::util::{rand_string, read_chain_config, write_toml};
use clap::Parser;

/// A subcommand for run
#[derive(Parser, Debug, Clone)]
pub struct SetNodeListOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub(crate) chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub(crate) config_dir: String,
    /// node list looks like localhost:40000:node0:k8s_cluster1,localhost:40001:node1:k8s_cluster2
    /// last slice is optional, none means not k8s env.
    #[clap(long = "nodelist")]
    pub(crate) node_list: String,
}

/// execute set node list
pub fn execute_set_nodelist(opts: SetNodeListOpts) -> Result<(), Error> {
    // load chain_config
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CHAIN_CONFIG_FILE
    );
    let mut chain_config = read_chain_config(&file_name).unwrap();

    // public and finalize is ok
    if chain_config.stage == ConfigStage::Init {
        return Err(Error::InvalidStage);
    }

    let node_list_str: Vec<&str> = opts.node_list.split(',').collect();
    let node_list = node_list_str
        .iter()
        .map(|node| {
            let node_network_info: Vec<&str> = node.split(':').collect();
            let cluster = if node_network_info.len() == 3 {
                rand_string()
            } else {
                node_network_info[3].to_string()
            };

            NodeNetworkAddressBuilder::new()
                .host(node_network_info[0].to_string())
                .port(node_network_info[1].parse::<u16>().unwrap())
                .domain(node_network_info[2].to_string())
                .cluster(cluster)
                .build()
        })
        .collect();

    chain_config.set_node_network_address_list(node_list);

    // store chain_config
    write_toml(&chain_config, file_name);

    Ok(())
}
