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
use crate::util::{cert, read_file, restore_ca_cert, write_file};
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

/// execute create cert
pub fn execute_create_cert(opts: CreateCertOpts) -> Result<(String, String), Error> {
    // load ca cert
    let ca_cert_path = format!("{}/{}/ca_cert/cert.pem", &opts.config_dir, &opts.chain_name);
    let ca_cert_pem = read_file(ca_cert_path).unwrap();

    let ca_key_path = format!("{}/{}/ca_cert/key.pem", &opts.config_dir, &opts.chain_name);
    let ca_key_pem = read_file(ca_key_path).unwrap();
    let ca = restore_ca_cert(&ca_cert_pem, &ca_key_pem);

    // gen cert of node by domain
    let (_, cert_pem, key_pem) = cert(&opts.domain, &ca);

    // gen a folder to store cert info
    let path = format!(
        "{}/{}/certs/{}",
        &opts.config_dir, &opts.chain_name, &opts.domain
    );
    fs::create_dir_all(&path).unwrap();

    let cert_pem_path = format!("{}/cert.pem", &path);
    write_file(cert_pem.as_bytes(), cert_pem_path);

    let key_pem_path = format!("{}/key.pem", &path);
    write_file(key_pem.as_bytes(), key_pem_path);

    Ok((cert_pem, key_pem))
}
