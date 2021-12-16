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
use crate::util::{touch_file, write_file};
use clap::Clap;
use std::fs;
use std::path::Path;
use crate::constant::{ACCOUNT_DIR, CA_CERT_DIR, CERTS_DIR, KEY_PEM};

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
/// --------  .gitkeep
/// ------  ca_cert
/// --------  .gitkeep
/// ------  certs
/// --------  .gitkeep
/// ------  .gitignore
pub fn execute_init_chain(opts: InitChainOpts) -> Result<(), Error> {
    let chain_path = format!("{}/{}", &opts.config_dir, &opts.chain_name);
    if Path::new(&chain_path).exists() {
        return Err(Error::DupChainName);
    }

    let path = format!("{}/{}", &chain_path, ACCOUNT_DIR);
    fs::create_dir_all(&path).unwrap();
    let gitkeep_path = format!("{}/{}/.gitkeep", &chain_path, ACCOUNT_DIR);
    touch_file(gitkeep_path);

    let path = format!("{}/{}", &chain_path, CERTS_DIR);
    fs::create_dir_all(&path).unwrap();
    let gitkeep_path = format!("{}/{}/.gitkeep", &chain_path, CERTS_DIR);
    touch_file(gitkeep_path);

    let path = format!("{}/{}", &chain_path, CA_CERT_DIR);
    fs::create_dir_all(&path).unwrap();
    let gitkeep_path = format!("{}/{}/.gitkeep", &chain_path, CA_CERT_DIR);
    touch_file(gitkeep_path);

    let git_ignore_path = format!("{}/.gitignore", &chain_path);
    let git_ignore_content = format!("{}/*/\n{}/{}\n{}/*/{}\n", ACCOUNT_DIR, CA_CERT_DIR, KEY_PEM, CERTS_DIR, KEY_PEM) ;
    write_file(git_ignore_content.as_bytes(), git_ignore_path);
    Ok(())
}
