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

use crate::config::chain_config::ConfigStage;
use crate::constant::CHAIN_CONFIG_FILE;
use crate::error::Error;
use crate::util::{read_chain_config, write_toml};
use clap::Parser;

/// A subcommand for run
#[derive(Parser, Debug, Clone)]
pub struct DeleteValidatorOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub(crate) chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub(crate) config_dir: String,
    /// validator account
    #[clap(long = "validator")]
    pub(crate) validator: String,
}

/// execute delete validator
pub fn execute_delete_validator(opts: DeleteValidatorOpts) -> Result<(), Error> {
    // load chain_config
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CHAIN_CONFIG_FILE
    );
    let mut chain_config = read_chain_config(&file_name).unwrap();

    if chain_config.stage != ConfigStage::Public {
        return Err(Error::InvalidStage);
    }

    let mut validators = chain_config.system_config.validators.clone();

    let mut pos_opt = None;
    for (pos, validator) in validators.iter().enumerate() {
        if validator.eq(&opts.validator) {
            pos_opt = Some(pos);
            break;
        }
    }

    match pos_opt {
        Some(pos) => {
            validators.remove(pos);
        }
        None => panic!(
            "Can't found validator that want to delete! validator {} not in {:?}",
            opts.validator, validators
        ),
    }

    chain_config.set_validators(validators);

    // store chain_config
    write_toml(&chain_config, file_name);

    Ok(())
}
