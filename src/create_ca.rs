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

use crate::constant::{CA_CERT_DIR, CERT_PEM, KEY_PEM};
use crate::error::Error;
use crate::util::{ca_cert, write_file};
use clap::Parser;

/// A subcommand for run
#[derive(Parser, Debug, Clone)]
pub struct CreateCAOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub config_dir: String,
}

/// execute create ca
pub fn execute_create_ca(opts: CreateCAOpts) -> Result<(String, String), Error> {
    let (_, ca_cert_pem, ca_key_pem) = ca_cert();

    let path = format!(
        "{}/{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CA_CERT_DIR, CERT_PEM
    );
    write_file(ca_cert_pem.as_bytes(), path);

    let path = format!(
        "{}/{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CA_CERT_DIR, KEY_PEM
    );
    write_file(ca_key_pem.as_bytes(), path);

    Ok((ca_cert_pem, ca_key_pem))
}
