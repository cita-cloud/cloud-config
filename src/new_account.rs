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

use crate::constant::{
    ACCOUNT_DIR, CHAIN_CONFIG_FILE, CONSENSUS_OVERLORD, KEY_ID, KMS_DB, KMS_ETH, PRIVATE_KEY,
    VALIDATOR_ADDRESS,
};
use crate::error::Error;
use crate::util::{find_micro_service, key_pair_option, read_chain_config, write_file};
use clap::Parser;
use ophelia::{PrivateKey, PublicKey, ToBlsPublicKey};
use ophelia_blst::BlsPrivateKey;
use rand::rngs::OsRng;
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
pub fn execute_new_account(opts: NewAccountOpts) -> Result<(u64, String, String), Error> {
    // load chain_config
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CHAIN_CONFIG_FILE
    );
    let chain_config = read_chain_config(&file_name).unwrap();

    // new account in base folder
    let base_path = format!("{}/{}/{}", &opts.config_dir, &opts.chain_name, ACCOUNT_DIR);

    let is_eth = find_micro_service(&chain_config, KMS_ETH);
    let (key_id, address) = key_pair_option(&base_path, opts.kms_password, is_eth);
    let address = hex::encode(address);

    // gen a folder to store account info
    let path = format!("{}/{}", &base_path, &address);
    fs::create_dir_all(&path).unwrap();

    // move account files info account folder
    let from = format!("{}/{}", &base_path, KMS_DB);
    let to = format!("{}/{}/{}", &base_path, &address, KMS_DB);
    fs::rename(from, to).unwrap();

    // store key_id
    let path = format!("{}/{}/{}", &base_path, &address, KEY_ID);
    write_file(format!("{}", key_id).as_bytes(), path);

    let is_overlord = find_micro_service(&chain_config, CONSENSUS_OVERLORD);
    let validator_address = if is_overlord {
        let private_key = BlsPrivateKey::generate(&mut OsRng);
        let common_ref = "".to_string();
        let pub_key = private_key.pub_key(&common_ref);
        let bls_address = pub_key.to_bytes().to_vec();
        let validator_address = hex::encode(bls_address);
        // store private_key
        let path = format!("{}/{}/{}", &base_path, address, PRIVATE_KEY);
        write_file(hex::encode(&private_key.to_bytes()).as_bytes(), path);
        // store validator_address
        let path = format!("{}/{}/{}", &base_path, address, VALIDATOR_ADDRESS);
        write_file(validator_address.as_bytes(), path);
        validator_address
    } else {
        address.clone()
    };

    // output key_id and address of new account
    println!(
        "key_id: {} node address: {} validator address: {}",
        key_id, address, validator_address
    );

    Ok((key_id, address, validator_address))
}
