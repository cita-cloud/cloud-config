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

use crate::constant::{ACCOUNT_DIR, CHAIN_CONFIG_FILE, KEY_ID, KMS_DB, KMS_ETH};
use crate::error::Error;
use crate::util::{find_micro_service, key_pair_option, read_chain_config, write_file};
use clap::Parser;
use std::fs;

/// A subcommand for run
#[derive(Parser, Debug, Clone)]
pub struct NewAccountOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub(crate) chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub(crate) config_dir: String,
    /// kms db password
    #[clap(long = "kms-password", default_value = "123456")]
    pub(crate) kms_password: String,
}

/// execute new account
pub fn execute_new_account(opts: NewAccountOpts) -> Result<(u64, String), Error> {
    // load chain_config
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CHAIN_CONFIG_FILE
    );
    let _chain_config = read_chain_config(&file_name).unwrap();

    // Now only support kms_sm
    let is_eth = find_micro_service(&_chain_config, KMS_ETH);

    // new account in base folder
    let base_path = format!("{}/{}/{}", &opts.config_dir, &opts.chain_name, ACCOUNT_DIR);
    let (key_id, address) = key_pair_option(base_path, opts.kms_password, is_eth);
    let address = hex::encode(address);

    // gen a folder to store account info
    let path = format!(
        "{}/{}/{}/{}",
        &opts.config_dir, &opts.chain_name, ACCOUNT_DIR, &address
    );
    fs::create_dir_all(&path).unwrap();

    // move account files info account folder
    let from = format!(
        "{}/{}/{}/{}",
        &opts.config_dir, &opts.chain_name, ACCOUNT_DIR, KMS_DB
    );
    let to = format!(
        "{}/{}/{}/{}/{}",
        &opts.config_dir, &opts.chain_name, ACCOUNT_DIR, &address, KMS_DB
    );
    fs::rename(from, to).unwrap();
    // store key_id
    let path = format!(
        "{}/{}/{}/{}/{}",
        &opts.config_dir, &opts.chain_name, ACCOUNT_DIR, &address, KEY_ID
    );
    write_file(format!("{}", key_id).as_bytes(), path);

    // output key_id and address of new account
    println!("key_id:{}, address:{}", key_id, address);

    Ok((key_id, address))
}
