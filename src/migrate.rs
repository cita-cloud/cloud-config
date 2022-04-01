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

use clap::{Parser, ValueHint};

use anyhow::{bail, ensure, Context, Result};

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;

use serde::de::DeserializeOwned;

use std::fs;

use crate::{
    create_ca::{execute_create_ca, CreateCAOpts},
    create_csr::{execute_create_csr, CreateCSROpts},
    import_account::{execute_import_account, ImportAccountOpts},
    init_chain::{execute_init_chain, InitChainOpts},
    init_chain_config::{execute_init_chain_config, InitChainConfigOpts},
    init_node::{execute_init_node, InitNodeOpts},
    set_admin::{execute_set_admin, SetAdminOpts},
    set_nodelist::{execute_set_nodelist, SetNodeListOpts},
    set_stage::{execute_set_stage, SetStageOpts},
    set_validators::{execute_set_validators, SetValidatorsOpts},
    sign_csr::{execute_sign_csr, SignCSROpts},
    update_node::{execute_update_node, UpdateNodeOpts},
    util::remove_0x,
};

const DEFAULT_PACKAGE_LIMIT: u64 = 30000;
const DEFAULT_BLOCK_LIMIT: u64 = 100;

mod old {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct ConsensusConfig {
        pub controller_port: u16,
    }

    #[derive(Deserialize)]
    pub struct ControllerConfig {
        pub network_port: u16,
        pub consensus_port: u16,
        pub storage_port: u16,
        pub kms_port: u16,
        pub executor_port: u16,
    }

    #[derive(Deserialize)]
    pub struct NetworkConfig {
        pub port: u16,
        pub peers: Vec<PeerConfig>,
    }

    #[derive(Deserialize)]
    pub struct PeerConfig {
        pub ip: String,
        pub port: u16,
    }

    #[derive(Deserialize, Clone)]
    pub struct InitSysConfig {
        pub version: u32,
        pub admin: String,
        pub block_interval: u32,
        pub chain_id: String,
        pub validators: Vec<String>,
    }

    #[derive(Deserialize, Clone)]
    pub struct Genesis {
        pub timestamp: u64,
        pub prevhash: String,
    }

    pub struct NodeConfig {
        // node config loaded from old

        // ports
        pub controller_port: u16,
        pub consensus_port: u16,
        pub executor_port: u16,
        pub network_port: u16,
        pub kms_port: u16,
        pub storage_port: u16,

        // controller
        pub node_addr: String,
        pub genesis_block: Genesis,
        pub system_config: InitSysConfig,

        // kms
        pub node_key: String,

        // network
        pub network_config: NetworkConfig,
    }

    impl NodeConfig {
        pub fn extract_from(data_dir: impl AsRef<Path>) -> Result<Self> {
            let ControllerConfig {
                consensus_port,
                storage_port,
                network_port,
                executor_port,
                kms_port,
            } = extract_toml(&data_dir, "controller-config.toml")?;

            let ConsensusConfig { controller_port } =
                extract_toml(&data_dir, "consensus-config.toml")?;

            let network_config: NetworkConfig = extract_toml(&data_dir, "network-config.toml")?;
            let node_addr = extract_text(&data_dir, "node_address")?;

            let system_config: InitSysConfig = extract_toml(&data_dir, "init_sys_config.toml")?;
            let genesis_block: Genesis = extract_toml(&data_dir, "genesis.toml")?;

            let node_key = extract_text(&data_dir, "node_key")?;

            let this = Self {
                controller_port,
                consensus_port,
                executor_port,
                network_port,
                kms_port,
                storage_port,

                // controller
                node_addr,
                genesis_block,
                system_config,

                // kms
                node_key,

                // network
                network_config,
            };

            Ok(this)
        }
    }

    fn extract_toml<T: DeserializeOwned>(data_dir: impl AsRef<Path>, file_name: &str) -> Result<T> {
        let s = extract_text(data_dir, file_name).context("cannot load toml file")?;
        let res: T = toml::from_str(&s).with_context(|| {
            format!("invalid toml for the `{}` type", std::any::type_name::<T>())
        })?;
        Ok(res)
    }

    fn extract_text(data_dir: impl AsRef<Path>, file_name: &str) -> Result<String> {
        let path = data_dir.as_ref().join(file_name);
        let mut f = File::open(&path).with_context(|| {
            format!(
                "cannot open file `{}` in `{}`",
                file_name,
                data_dir.as_ref().display()
            )
        })?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)
            .with_context(|| format!("cannot read data from {}", path.display()))?;
        Ok(buf)
    }
}

pub fn extract_old_node_configs(
    chain_data_dir: impl AsRef<Path>,
    chain_name: &str,
) -> Result<Vec<old::NodeConfig>> {
    let chain_data_dir = chain_data_dir.as_ref();
    ensure!(chain_data_dir.is_dir(), "chain data folder not found");

    // Load node dirs.
    let mut node_dirs: Vec<PathBuf> = fs::read_dir(chain_data_dir)
        .unwrap()
        .filter_map(|ent| {
            let ent = ent.unwrap();
            let dir_name = ent.file_name().into_string().unwrap();
            let prefix = format!("{}-", chain_name);
            if ent.file_type().unwrap().is_dir()
                && dir_name.starts_with(&prefix)
                && dir_name
                    .strip_prefix(&prefix)
                    .unwrap()
                    .parse::<u64>()
                    .is_ok()
            {
                Some(ent.path())
            } else {
                None
            }
        })
        .collect();

    // Sort node dirs according to their node_id.
    node_dirs.sort_by_key(|d| {
        let dir_name = d.file_name().unwrap().to_string_lossy();
        let node_id: u64 = dir_name
            .strip_prefix(&format!("{}-", chain_name))
            .unwrap()
            .parse()
            .unwrap();
        node_id
    });

    node_dirs
        .iter()
        .map(|d| {
            old::NodeConfig::extract_from(d)
                .with_context(|| format!("cannot extract old node config from `{}`", d.display()))
        })
        .collect::<Result<Vec<old::NodeConfig>>>()
}

fn generate_new_node_config(
    chain_name: &str,
    config_dir: &str,
    domain: &str,
    kms_password: &str,
    old_config: old::NodeConfig,
) -> Result<()> {
    let old::NodeConfig {
        controller_port,
        consensus_port,
        executor_port,
        network_port,
        kms_port,
        storage_port,

        node_addr,
        node_key,

        network_config,
        ..
    } = old_config;
    let node_addr = remove_0x(&node_addr).to_string();

    let (key_id, account_addr) = execute_import_account(ImportAccountOpts {
        chain_name: chain_name.into(),
        config_dir: config_dir.into(),
        kms_password: kms_password.into(),
        privkey: node_key,
    })
    .context("cannot import account")?;
    assert_eq!(
        node_addr, account_addr,
        "node address differs from account address"
    );

    execute_create_csr(CreateCSROpts {
        chain_name: chain_name.into(),
        config_dir: config_dir.into(),
        domain: domain.into(),
    })
    .unwrap();
    execute_sign_csr(SignCSROpts {
        chain_name: chain_name.into(),
        config_dir: config_dir.into(),
        domain: domain.into(),
    })
    .unwrap();

    execute_init_node(InitNodeOpts {
        chain_name: chain_name.into(),
        config_dir: config_dir.into(),

        controller_port,
        consensus_port,
        executor_port,
        network_port,
        kms_port,
        storage_port,

        network_listen_port: network_config.port,

        key_id,
        kms_password: kms_password.into(),
        account: node_addr,

        domain: domain.into(),
        package_limit: DEFAULT_PACKAGE_LIMIT,

        log_level: "info".into(),

        cluster: "".to_string(),
    })
    .unwrap();

    execute_update_node(UpdateNodeOpts {
        chain_name: chain_name.into(),
        config_dir: config_dir.into(),
        config_name: "config.toml".into(),
        domain: domain.into(),
        is_stdout: false,
        is_old: false,
    })
    .unwrap();

    Ok(())
}

pub fn migrate<P, Q>(
    chain_data_dir: P,
    new_chain_data_dir: Q,
    chain_name: &str,
    consensus_type: ConsensusType,
    kms_password_list: Vec<String>,
    node_list: Vec<(String, u16)>,
) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let node_configs = extract_old_node_configs(chain_data_dir.as_ref(), chain_name)
        .context("cannot extract node configs")?;
    ensure!(!node_configs.is_empty(), "No node config found",);
    ensure!(
        node_configs.len() == node_list.len(),
        "The length of node configs and node list mismatched"
    );
    ensure!(
        node_configs.len() == kms_password_list.len(),
        "The length of node configs and kms password list mismatched"
    );

    let config_dir = &new_chain_data_dir.as_ref().to_string_lossy().to_string();

    let sample = node_configs.first().unwrap();
    let old::InitSysConfig {
        version,
        admin,
        chain_id,
        block_interval,
        validators,
    } = sample.system_config.clone();
    let admin = remove_0x(&admin).to_string();
    let chain_id = remove_0x(&chain_id).to_string();
    let validators = validators
        .iter()
        .map(|v| remove_0x(v))
        .collect::<Vec<&str>>()
        .join(",");

    let old::Genesis {
        timestamp,
        prevhash,
    } = sample.genesis_block.clone();

    let consensus_image = match consensus_type {
        ConsensusType::Raft => "consensus_raft".into(),
        ConsensusType::Bft => "consensus_bft".into(),
    };

    execute_init_chain(InitChainOpts {
        chain_name: chain_name.into(),
        config_dir: config_dir.into(),
    })
    .unwrap();

    execute_init_chain_config(InitChainConfigOpts {
        chain_name: chain_name.into(),
        config_dir: config_dir.into(),
        timestamp,
        prevhash,
        chain_id,
        block_interval,
        version,
        block_limit: DEFAULT_BLOCK_LIMIT,

        controller_image: "controller".into(),
        controller_tag: "6.3.0".into(),

        network_image: "network_tls".into(),
        network_tag: "6.3.0".into(),

        consensus_image,
        consensus_tag: "6.3.0".into(),

        executor_image: "executor_evm".into(),
        executor_tag: "6.3.0".into(),

        storage_image: "storage_rocksdb".into(),
        storage_tag: "6.3.0".into(),

        kms_image: "kms_sm".into(),
        kms_tag: "6.3.0".into(),
    })
    .unwrap();

    execute_set_admin(SetAdminOpts {
        chain_name: chain_name.into(),
        config_dir: config_dir.into(),
        admin,
    })
    .unwrap();

    execute_set_validators(SetValidatorsOpts {
        chain_name: chain_name.into(),
        config_dir: config_dir.into(),
        validators,
    })
    .unwrap();

    let node_list = node_list
        .into_iter()
        .zip(0..node_configs.len())
        .map(|((ip, port), idx)| format!("{}:{}:{}", ip, port, idx))
        .collect::<Vec<String>>()
        .join(",");
    execute_set_nodelist(SetNodeListOpts {
        chain_name: chain_name.into(),
        config_dir: config_dir.into(),
        node_list,
    })
    .unwrap();

    execute_create_ca(CreateCAOpts {
        chain_name: chain_name.into(),
        config_dir: config_dir.into(),
    })
    .unwrap();

    execute_set_stage(SetStageOpts {
        chain_name: chain_name.into(),
        config_dir: config_dir.into(),
        stage: "finalize".into(),
    })
    .unwrap();

    for (i, (old_config, kms_password)) in
        node_configs.into_iter().zip(kms_password_list).enumerate()
    {
        let domain = i.to_string();
        generate_new_node_config(chain_name, config_dir, &domain, &kms_password, old_config)
            .context("cannot generate new node config")?;
    }

    Ok(())
}

#[derive(Clone, Copy)]
pub enum ConsensusType {
    Raft,
    Bft,
}

/// Migrate CITA-Cloud chain from 6.1.0 to 6.3.0
/// WARNING:
/// This is for a very specific use case, other cases may not work!
/// DO NOT use it if you don't know what is it for.
/// Back up your data before use it.
#[derive(Parser, Debug, Clone)]
pub struct MigrateOpts {
    /// The old chain dir
    #[clap(short = 'd', long = "chain-dir", value_hint = ValueHint::DirPath)]
    pub chain_dir: String,
    /// The output dir for the upgraded chain
    #[clap(short = 'o', long = "out-dir", value_hint = ValueHint::DirPath)]
    pub out_dir: String,
    /// Name of the chain
    #[clap(short = 'n', long = "chain-name")]
    pub chain_name: String,
    /// Consensus type, only `raft` or `bft` is supported
    #[clap(short = 'c', long = "consensus-type", default_value = "raft")]
    pub consensus_type: String,

    /// KMS password list, e.g. "node1password,node2password"
    #[clap(short = 'k', long = "kms-password-list")]
    pub kms_password_list: String,
    /// Node list, e.g. "127.0.0.1:40000,citacloud.org:40001"
    #[clap(short = 'l', long = "nodelist")]
    pub node_list: String,
}

pub fn execute_migrate(opts: MigrateOpts) -> Result<()> {
    let chain_dir = opts.chain_dir;
    let out_dir = opts.out_dir;
    let chain_name = opts.chain_name;
    let consensus_type = match opts.consensus_type.to_ascii_lowercase().as_str() {
        "raft" => ConsensusType::Raft,
        "bft" => ConsensusType::Bft,
        _ => bail!("unkown consensus type, possible values are [`raft`, `bft`]"),
    };

    let kms_password_list: Vec<String> = opts
        .kms_password_list
        .split(',')
        .map(String::from)
        .collect();
    let node_list: Vec<(String, u16)> = opts
        .node_list
        .split(',')
        .map(|s| {
            let (ip, port) = s.split_once(':').context("cannot parse ip and port")?;
            Ok((ip.to_string(), port.parse()?))
        })
        .collect::<Result<Vec<(String, u16)>>>()
        .context("cannot parse node list")?;

    migrate(
        &chain_dir,
        &out_dir,
        &chain_name,
        consensus_type,
        kms_password_list,
        node_list,
    )
    .context("cannot migrate chain")?;

    println!("Finshed. new config write to `{}`", out_dir);
    Ok(())
}
