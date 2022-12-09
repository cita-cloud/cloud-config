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
    ACCOUNT_DIR, CHAIN_CONFIG_FILE, CONSENSUS_OVERLORD, CRYPTO_ETH, NODE_ADDRESS, PRIVATE_KEY,
    VALIDATOR_ADDRESS,
};
use crate::error::Error;
use crate::util::{find_micro_service, read_chain_config, write_file};
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
}

/// execute new account
pub fn execute_new_account(opts: NewAccountOpts) -> Result<(String, String), Error> {
    // load chain_config
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CHAIN_CONFIG_FILE
    );
    let chain_config = read_chain_config(&file_name).unwrap();

    // create new account
    // generate private key
    let private_key = BlsPrivateKey::generate(&mut OsRng).to_bytes();

    // generate node_address
    let is_eth = find_micro_service(&chain_config, CRYPTO_ETH);
    let address = if is_eth {
        crypto_eth::eth::sk2address(private_key.as_ref())
    } else {
        crypto_sm::sm::sk2address(private_key.as_ref())
    };
    let address = hex::encode(address);

    // gen a folder to store account info
    let base_path = format!("{}/{}/{}", &opts.config_dir, &opts.chain_name, ACCOUNT_DIR);
    let path = format!("{}/{}", &base_path, &address);
    fs::create_dir_all(&path).unwrap();

    // store private_key
    let path = format!("{}/{}/{}", &base_path, address, PRIVATE_KEY);
    write_file(hex::encode(&private_key).as_bytes(), path);

    let is_overlord = find_micro_service(&chain_config, CONSENSUS_OVERLORD);
    let validator_address = if is_overlord {
        let private_key = BlsPrivateKey::try_from(private_key.as_ref()).unwrap();
        let common_ref = "".to_string();
        let pub_key = private_key.pub_key(&common_ref);
        let bls_address = pub_key.to_bytes().to_vec();
        hex::encode(bls_address)
    } else {
        address.clone()
    };

    // store validator_address
    let path = format!("{}/{}/{}", &base_path, address, VALIDATOR_ADDRESS);
    write_file(validator_address.as_bytes(), path);

    // store node_address
    let path = format!("{}/{}/{}", &base_path, address, NODE_ADDRESS);
    write_file(address.as_bytes(), path);

    // output node_address and validator_address
    println!(
        "node_address: {} validator_address: {}",
        address, validator_address
    );

    Ok((address, validator_address))
}
