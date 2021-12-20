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

use crate::constant::CHAIN_CONFIG_FILE;
use crate::error::Error;
use crate::util::{read_chain_config, write_toml, check_address};
use clap::Clap;

/// A subcommand for run
#[derive(Clap, Debug, Clone)]
pub struct AppendValidatorOpts {
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

/// execute set validators
pub fn execute_append_validator(opts: AppendValidatorOpts) -> Result<(), Error> {
    // load chain_config
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CHAIN_CONFIG_FILE
    );
    let mut chain_config = read_chain_config(&file_name).unwrap();

    let mut validators = chain_config.system_config.validators.clone();

    validators.push(check_address(&opts.validator[..]).to_string());

    chain_config.set_validators(validators);

    // store chain_config
    write_toml(&chain_config, file_name);

    Ok(())
}
