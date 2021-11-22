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
use crate::util::{read_chain_config, read_node_config};
use clap::Clap;

/// A subcommand for run
#[derive(Clap, Debug, Clone)]
pub struct UpdateNodeOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    config_dir: String,
    /// domain of node
    #[clap(long = "domain")]
    domain: String,
}

/// generate node config files by chain_config and node_config
pub fn execute_update_node(opts: UpdateNodeOpts) -> Result<(), Error> {
    // load chain_config
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, "chain_config.toml"
    );
    let mut chain_config = read_chain_config(&file_name).unwrap();

    // load node_config
    let file_name = format!(
        "{}/{}-{}/node_config.toml",
        &opts.config_dir, &opts.chain_name, &opts.domain
    );
    let node_config = read_node_config(file_name).unwrap();

    // TODO: generate node config files

    Ok(())
}
