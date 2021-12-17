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

use std::fs;
use std::path::Path;

use clap::Clap;

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;

use crate::{
    config::{kms_eth::KmsEth, kms_sm::KmsSm},
    constant::{ACCOUNT_DIR, CHAIN_CONFIG_FILE, KEY_ID, KMS_DB, KMS_ETH, KMS_SM},
    traits::Kms,
    util::{find_micro_service, read_chain_config, write_file},
};

/// A subcommand for import account, only kms_sm is supported
#[derive(Clap, Debug, Clone)]
pub struct ImportAccountOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub(crate) chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub(crate) config_dir: String,
    /// kms db password
    #[clap(long = "kms-password", default_value = "123456")]
    pub(crate) kms_password: String,
    /// hex encoded private key
    #[clap(long = "privkey")]
    pub(crate) privkey: String,
}

// return key_id and address of the account
fn import_account<K: Kms, P: AsRef<Path>>(
    base_dir: P,
    kms_password: &str,
    privkey: &str,
) -> Result<(u64, Vec<u8>)> {
    let privkey = {
        let s = crate::util::remove_0x(privkey);
        hex::decode(s).context("invalid `node_key`")?
    };

    let account_dir = {
        let addr = hex::encode(K::sk2address(&privkey));
        base_dir.as_ref().join(&addr)
    };
    fs::create_dir_all(&account_dir).context("cannot create account dir")?;

    let db_path = account_dir.join(KMS_DB).to_string_lossy().into();
    let kms = K::create_kms_db(db_path, kms_password.into());
    let (key_id, addr) = kms.import_privkey(&privkey);

    let key_id_path = account_dir.join(KEY_ID);
    write_file(key_id.to_string().as_bytes(), key_id_path);

    Ok((key_id, addr))
}

pub fn execute_import_account(opts: ImportAccountOpts) -> Result<(u64, String)> {
    // load chain_config
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CHAIN_CONFIG_FILE
    );
    let chain_config = read_chain_config(&file_name).unwrap();

    let base_dir = format!("{}/{}/{}", &opts.config_dir, &opts.chain_name, ACCOUNT_DIR);
    let (key_id, addr) = {
        let import_account = if find_micro_service(&chain_config, KMS_ETH) {
            import_account::<KmsEth, _>
        } else if find_micro_service(&chain_config, KMS_SM) {
            import_account::<KmsSm, _>
        } else {
            bail!("unknown kms type");
        };
        import_account(base_dir, &opts.kms_password, &opts.privkey)
            .context("cannot import account")?
    };

    let addr = hex::encode(addr);
    println!("key_id:{}, address:{}", key_id, addr);
    Ok((key_id, addr))
}
