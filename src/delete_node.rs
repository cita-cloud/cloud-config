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
use crate::util::{read_chain_config, write_toml, read_from_file};
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
pub fn execute_set_node_list(opts: DeleteNodeOpts) {
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
}

pub fn execute_delete_folder(opts: DeleteNodeOpts)  {
    //delete node folder
    let path = format!("{}/{}-{}", &opts.config_dir, &opts.chain_name, &opts.domain);

    //delete account folder
    let config = format!("{}/{}", &path, &opts.config_name);
    let config_toml = read_from_file(&config).unwrap();
    let account_path =  format!("{}/{}/accounts/{}", &opts.config_dir, &opts.chain_name, &config_toml.controller.unwrap().node_address);
    fs::remove_dir_all(&account_path).unwrap();
    fs::remove_dir_all(&path).unwrap();


    //delete cert folder
    let cert_path = format!("{}/{}/certs/{}", &opts.config_dir, &opts.chain_name, &opts.domain);
    fs::remove_dir_all(&cert_path).unwrap();
}



/// execute set node list
pub fn execute_delete_node(opts: DeleteNodeOpts) -> Result<(), Error> {

    execute_set_node_list(opts.clone());
    execute_delete_folder(opts);

    Ok(())

}
