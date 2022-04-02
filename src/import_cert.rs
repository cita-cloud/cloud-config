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

use crate::constant::{CERTS_DIR, CERT_PEM, KEY_PEM};
use crate::error::Error;
use crate::util::read_file;
use clap::Parser;
use std::fs;
use std::path::Path;

/// A subcommand for run
#[derive(Parser, Debug, Clone)]
pub struct ImportCertOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub(crate) chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub(crate) config_dir: String,
    /// domain of node
    #[clap(long = "domain")]
    pub(crate) domain: String,
    /// set path of cert file(pem)
    #[clap(long = "cert")]
    pub(crate) cert_path: String,
    /// set path of key file(pem)
    #[clap(long = "key")]
    pub(crate) key_path: String,
}

/// execute create csr
pub fn execute_import_cert(opts: ImportCertOpts) -> Result<(String, String), Error> {
    if !Path::new(&opts.cert_path).exists() {
        return Err(Error::FileNoFound);
    }

    if !Path::new(&opts.key_path).exists() {
        return Err(Error::FileNoFound);
    }

    // gen a folder to store cert info
    let path = format!(
        "{}/{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CERTS_DIR, &opts.domain
    );
    fs::create_dir_all(&path).unwrap();

    let cert_pem_path = format!("{}/{}", &path, CERT_PEM);
    fs::copy(&opts.cert_path, &cert_pem_path).unwrap();

    let key_pem_path = format!("{}/{}", &path, KEY_PEM);
    fs::copy(&opts.key_path, &key_pem_path).unwrap();

    let cert_pem = read_file(cert_pem_path).unwrap();
    let key_pem = read_file(key_pem_path).unwrap();

    Ok((cert_pem, key_pem))
}
