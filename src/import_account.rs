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

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;

use crate::{
    traits::Kms,
    util::write_file,
    config::{
        kms_eth::KmsEth,
        kms_sm::KmsSm,
    },
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
    /// kms type
    #[clap(long = "kms-type", default_value = "kms_sm")]
    pub(crate) kms_type: String,
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

    let db_path = account_dir.join("kms.db").to_string_lossy().into();
    let kms = K::create_kms_db(db_path, kms_password.into());
    let (key_id, addr) = kms.import_privkey(&privkey);

    let key_id_path = account_dir.join("key_id");
    write_file(key_id.to_string().as_bytes(), key_id_path);

    Ok((key_id, addr))
}

pub fn execute_import_account(opts: ImportAccountOpts) -> Result<(u64, String)> {
    let base_dir = format!("{}/{}/accounts", &opts.config_dir, &opts.chain_name);
    let (key_id, addr) = {
        let import_account = match opts.kms_type.as_str() {
            "kms_sm" => import_account::<KmsSm, _>,
            "kms_eth" => import_account::<KmsEth, _>,
            _ => bail!("unknown kms type"),
        };
        import_account(base_dir, &opts.kms_password, &opts.privkey)
            .context("cannot import account")?
    };

    let addr = hex::encode(addr);
    println!("key_id:{}, address:{}", key_id, addr);
    Ok((key_id, addr))
}
