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
use crate::util::read_chain_config;
use clap::Clap;
use std::fs;

/// A subcommand for run
#[derive(Clap, Debug, Clone)]
pub struct NewAccountOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    config_dir: String,
}

/// execute set admin
pub fn execute_new_account(opts: NewAccountOpts) -> Result<(), Error> {
    // load chain_config
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, "chain_config.toml"
    );
    let chain_config = read_chain_config(&file_name).unwrap();

    // TODO : check kms micro service name and gen account

    // gen a folder to store account info
    let path = format!(
        "{}/{}/accounts/{}",
        &opts.config_dir, &opts.chain_name, "0x014328c8df26a088c621e2f8ac034ff0aa21cffd"
    );
    fs::create_dir_all(&path).unwrap();

    // output address of new account
    println!("0x014328c8df26a088c621e2f8ac034ff0aa21cffd");

    Ok(())
}
