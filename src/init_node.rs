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

use crate::config::node_config::NodeConfig;
use crate::error::Error;
use crate::util::{read_chain_config, write_toml};
use clap::Clap;

/// A subcommand for run
#[derive(Clap, Debug, Clone)]
pub struct InitNodeOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    config_dir: String,
    /// domain of node
    #[clap(long = "domain")]
    domain: String,
    /// account of node
    #[clap(long = "account")]
    account: String,
}

/// execute set validators
pub fn execute_init_node(opts: InitNodeOpts) -> Result<(), Error> {
    let node_config = NodeConfig::new().build();

    // TODO: add more args

    let file_name = format!(
        "{}/{}-{}/node_config.toml",
        &opts.config_dir, &opts.chain_name, &opts.domain
    );
    write_toml(&node_config, file_name);

    Ok(())
}
