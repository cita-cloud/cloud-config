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

use crate::constant::{CA_CERT_DIR, CERTS_DIR, CERT_PEM, CSR_PEM, KEY_PEM};
use crate::error::Error;
use crate::util::{read_file, sign_csr, write_file};
use clap::Parser;

/// A subcommand for run
#[derive(Parser, Debug, Clone)]
pub struct SignCSROpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub config_dir: String,
    /// domain of node
    #[clap(long = "domain")]
    pub domain: String,
}

/// execute sign csr
pub fn execute_sign_csr(opts: SignCSROpts) -> Result<String, Error> {
    // load ca cert
    let ca_cert_path = format!(
        "{}/{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CA_CERT_DIR, CERT_PEM
    );
    let ca_cert_pem = read_file(ca_cert_path).unwrap();

    let ca_key_path = format!(
        "{}/{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CA_CERT_DIR, KEY_PEM
    );
    let ca_key_pem = read_file(ca_key_path).unwrap();

    // load csr
    let csr_pem_path = format!(
        "{}/{}/{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CERTS_DIR, &opts.domain, CSR_PEM
    );
    let csr_pem = read_file(csr_pem_path).unwrap();

    // sign csr
    let cert_pem = sign_csr(&csr_pem, &ca_cert_pem, &ca_key_pem);

    let cert_pem_path = format!(
        "{}/{}/{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CERTS_DIR, &opts.domain, CERT_PEM
    );
    write_file(cert_pem.as_bytes(), cert_pem_path);

    Ok(cert_pem)
}
