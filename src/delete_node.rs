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

use crate::constant::{ACCOUNT_DIR, CERTS_DIR, CHAIN_CONFIG_FILE, NODE_CONFIG_FILE};
use crate::error::Error;
use crate::util::{read_chain_config, read_node_config, write_toml};
use clap::Clap;
use std::fs;

/// A subcommand for run
#[derive(Clap, Debug, Clone)]
pub struct DeleteNodeOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub(crate) chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub(crate) config_dir: String,
    /// set config file name
    #[clap(long = "config-name", default_value = "config.toml")]
    pub(crate) config_name: String,
    /// domain of node that want to delete
    #[clap(long = "domain")]
    pub(crate) domain: String,
}

pub fn execute_delete_node(opts: DeleteNodeOpts) -> Result<(), Error> {
    // load chain_config
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CHAIN_CONFIG_FILE
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

pub fn delete_node_folders(config_dir: &str, chain_name: &str, domain: &str) {
    let node_dir = format!("{}/{}-{}", config_dir, chain_name, domain);

    // load node_config
    let file_name = format!("{}/{}", &node_dir, NODE_CONFIG_FILE);
    let node_config = read_node_config(file_name).unwrap();

    // delete node folder
    fs::remove_dir_all(&node_dir).unwrap();

    // delete account folder
    let account_path = format!(
        "{}/{}/{}/{}",
        config_dir, chain_name, ACCOUNT_DIR, &node_config.account,
    );
    fs::remove_dir_all(&account_path).unwrap();

    // delete cert folder
    // ignore error because maybe cert folder doesn't exist
    let cert_path = format!("{}/{}/{}/{}", config_dir, chain_name, CERTS_DIR, domain);
    let _ = fs::remove_dir_all(&cert_path);
}
