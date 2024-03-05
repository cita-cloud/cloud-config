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
use crate::util::read_file;
use clap::Parser;
use std::fs;
use std::path::Path;

/// A subcommand for run
#[derive(Parser, Debug, Clone)]
pub struct ImportCAOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub config_dir: String,
    /// set path of ca cert file(pem)
    #[clap(long = "ca-cert")]
    pub ca_cert_path: String,
    /// set path of ca key file(pem)
    #[clap(long = "ca-key")]
    pub ca_key_path: String,
}

/// execute import ca
pub fn execute_import_ca(opts: ImportCAOpts) -> Result<(String, String), Error> {
    if !Path::new(&opts.ca_cert_path).exists() {
        return Err(Error::FileNoFound);
    }

    if !Path::new(&opts.ca_key_path).exists() {
        return Err(Error::FileNoFound);
    }

    let cert_path = format!(
        "{}/{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CA_CERT_DIR, CERT_PEM
    );

    fs::copy(&opts.ca_cert_path, &cert_path).unwrap();

    let key_path = format!(
        "{}/{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CA_CERT_DIR, KEY_PEM
    );

    fs::copy(&opts.ca_key_path, &key_path).unwrap();

    let ca_cert_pem = read_file(cert_path).unwrap();
    let ca_key_pem = read_file(key_path).unwrap();

    Ok((ca_cert_pem, ca_key_pem))
}
