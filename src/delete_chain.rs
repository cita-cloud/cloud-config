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

use crate::constant::CHAIN_CONFIG_FILE;
use crate::error::Error;
use crate::util::read_chain_config;
use clap::Parser;
use std::fs;

/// A subcommand for run
#[derive(Parser, Debug, Clone)]
pub struct DeleteChainOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub(crate) chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub(crate) config_dir: String,
}

/// delete these folders
/// $(config_dir)
/// --  $(chain_name)
/// --  $(chain_name)-xxx
pub fn execute_delete_chain(opts: DeleteChainOpts) -> Result<(), Error> {
    // load chain_config
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CHAIN_CONFIG_FILE
    );
    let chain_config = read_chain_config(&file_name).unwrap();

    // delete node folders
    for node in chain_config.node_network_address_list {
        let path = format!("{}/{}-{}", &opts.config_dir, &opts.chain_name, &node.domain);
        fs::remove_dir_all(&path).unwrap();
    }

    let path = format!("{}/{}", &opts.config_dir, &opts.chain_name);
    fs::remove_dir_all(&path).unwrap();

    Ok(())
}
