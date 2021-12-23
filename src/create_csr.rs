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

use crate::constant::{CERTS_DIR, CSR_PEM, KEY_PEM};
use crate::error::Error;
use crate::util::{create_csr, write_file};
use clap::Clap;
use std::fs;

/// A subcommand for run
#[derive(Clap, Debug, Clone)]
pub struct CreateCSROpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub(crate) chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub(crate) config_dir: String,
    /// domain of node
    #[clap(long = "domain")]
    pub(crate) domain: String,
}

/// execute create csr
pub fn execute_create_csr(opts: CreateCSROpts) -> Result<(String, String), Error> {
    // gen csr and key_pem of node by domain
    let real_domain = format!("{}-{}", &opts.chain_name, &opts.domain);
    let (csr_pem, key_pem) = create_csr(&real_domain);

    // gen a folder to store cert info
    let path = format!(
        "{}/{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CERTS_DIR, &opts.domain
    );
    fs::create_dir_all(&path).unwrap();

    let csr_pem_path = format!("{}/{}", &path, CSR_PEM);
    write_file(csr_pem.as_bytes(), csr_pem_path);

    let key_pem_path = format!("{}/{}", &path, KEY_PEM);
    write_file(key_pem.as_bytes(), key_pem_path);

    Ok((csr_pem, key_pem))
}
