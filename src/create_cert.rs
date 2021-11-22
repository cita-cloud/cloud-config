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

/// A subcommand for run
#[derive(Clap, Debug, Clone)]
pub struct CreateCertOpts {
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

/// execute set admin
pub fn execute_create_cert(opts: CreateCertOpts) -> Result<(), Error> {
    // TODO : load ca_cert and gen node cert then store it into certs folder

    // gen a folder to store account info
    let path = format!(
        "{}/{}/certs/{}",
        &opts.config_dir, &opts.chain_name, &opts.domain
    );
    fs::create_dir_all(&path).unwrap();

    Ok(())
}
