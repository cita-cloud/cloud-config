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

use crate::error::Error;
use crate::util::{read_chain_config, write_toml};
use clap::Clap;

/// A subcommand for run
#[derive(Clap, Debug, Clone)]
pub struct DeleteNodeOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    config_dir: String,
    /// domain of node that want to delete
    #[clap(long = "domain")]
    domain: String,
}

/// execute set node list
pub fn execute_delete_node(opts: DeleteNodeOpts) -> Result<(), Error> {
    // load chain_config
    let file_name = format!(
        "{}/{}/chain_config.toml",
        &opts.config_dir, &opts.chain_name
    );
    let mut chain_config = read_chain_config(&file_name).unwrap();

    let mut node_list = chain_config.node_network_address_list.clone();

    match node_list.binary_search_by(|node| node.domain.cmp(&opts.domain)) {
        Ok(index) => {
            node_list.remove(index);
        }
        Err(_) => panic!("Can't found node that want to delete!"),
    }

    chain_config.set_node_network_address_list(node_list);

    // store chain_config
    write_toml(&chain_config, file_name);

    Ok(())
}
