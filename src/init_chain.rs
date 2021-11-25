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
use clap::Clap;
use std::fs;
use std::path::Path;

/// A subcommand for run
#[derive(Clap, Debug, Clone)]
pub struct InitChainOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub(crate) chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub(crate) config_dir: String,
}

/// dir looks like
/// $(config_dir)
/// --  $(chain_name)
/// ------  accounts
/// ------  ca_cert
/// ------  certs
pub fn execute_init_chain(opts: InitChainOpts) -> Result<(), Error> {
    let chain_path = format!("{}/{}", &opts.config_dir, &opts.chain_name);
    if Path::new(&chain_path).exists() {
        return Err(Error::DupChainName);
    }

    let path = format!("{}/{}", &chain_path, "accounts");
    fs::create_dir_all(&path).unwrap();

    let path = format!("{}/{}", &chain_path, "certs");
    fs::create_dir_all(&path).unwrap();

    let path = format!("{}/{}", &chain_path, "ca_cert");
    fs::create_dir_all(&path).unwrap();
    Ok(())
}
