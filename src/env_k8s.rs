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

use crate::append_node::{execute_append_node, AppendNodeOpts};
use crate::append_validator::{execute_append_validator, AppendValidatorOpts};
use crate::config::chain_config::{NodeNetworkAddress, NodeNetworkAddressBuilder};
use crate::constant::{CHAIN_CONFIG_FILE, NETWORK_TLS};
use crate::create_ca::{execute_create_ca, CreateCAOpts};
use crate::create_csr::{execute_create_csr, CreateCSROpts};
use crate::delete_node::{delete_node_folders, execute_delete_node, DeleteNodeOpts};
use crate::error::Error;
use crate::init_chain::{execute_init_chain, InitChainOpts};
use crate::init_chain_config::{execute_init_chain_config, InitChainConfigOpts};
use crate::init_node::{execute_init_node, InitNodeOpts};
use crate::new_account::{execute_new_account, NewAccountOpts};
use crate::set_admin::{execute_set_admin, SetAdminOpts};
use crate::set_nodelist::{execute_set_nodelist, SetNodeListOpts};
use crate::set_stage::{execute_set_stage, SetStageOpts};
use crate::sign_csr::{execute_sign_csr, SignCSROpts};
use crate::update_node::{execute_update_node, UpdateNodeOpts};
use crate::util::{find_micro_service, rand_string, read_chain_config};
use clap::Parser;

/// A subcommand for run
#[derive(Parser, Debug, Clone)]
pub struct CreateK8sOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub(crate) chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub(crate) config_dir: String,
    /// set genesis timestamp
    #[clap(long = "timestamp", default_value = "0")]
    pub(crate) timestamp: u64,
    /// set genesis prevhash
    #[clap(
        long = "prevhash",
        default_value = "0x0000000000000000000000000000000000000000000000000000000000000000"
    )]
    pub(crate) prevhash: String,
    /// set system config version
    #[clap(long = "version", default_value = "0")]
    pub(crate) version: u32,
    /// set system config chain_id
    #[clap(long = "chain_id", default_value = "")]
    pub(crate) chain_id: String,
    /// set system config block_interval
    #[clap(long = "block_interval", default_value = "3")]
    pub(crate) block_interval: u32,
    /// set system config block_limit
    #[clap(long = "block_limit", default_value = "100")]
    pub(crate) block_limit: u64,
    /// set network micro service image name (network_tls/network_p2p)
    #[clap(long = "network_image", default_value = "network_tls")]
    pub(crate) network_image: String,
    /// set network micro service image tag
    #[clap(long = "network_tag", default_value = "latest")]
    pub(crate) network_tag: String,
    /// set consensus micro service image name (consensus_bft/consensus_raft/consensus_overlord)
    #[clap(long = "consensus_image", default_value = "consensus_bft")]
    pub(crate) consensus_image: String,
    /// set consensus micro service image tag
    #[clap(long = "consensus_tag", default_value = "latest")]
    pub(crate) consensus_tag: String,
    /// set executor micro service image name (executor_evm)
    #[clap(long = "executor_image", default_value = "executor_evm")]
    pub(crate) executor_image: String,
    /// set executor micro service image tag
    #[clap(long = "executor_tag", default_value = "latest")]
    pub(crate) executor_tag: String,
    /// set storage micro service image name (storage_rocksdb)
    #[clap(long = "storage_image", default_value = "storage_rocksdb")]
    pub(crate) storage_image: String,
    /// set storage micro service image tag
    #[clap(long = "storage_tag", default_value = "latest")]
    pub(crate) storage_tag: String,
    /// set controller micro service image name (controller)
    #[clap(long = "controller_image", default_value = "controller")]
    pub(crate) controller_image: String,
    /// set controller micro service image tag
    #[clap(long = "controller_tag", default_value = "latest")]
    pub(crate) controller_tag: String,
    /// set kms micro service image name (kms_eth/kms_sm)
    #[clap(long = "kms_image", default_value = "kms_sm")]
    pub(crate) kms_image: String,
    /// set kms micro service image tag
    #[clap(long = "kms_tag", default_value = "latest")]
    pub(crate) kms_tag: String,

    /// set admin
    #[clap(long = "admin")]
    pub(crate) admin: String,

    /// kms db password list, splited by ,
    #[clap(long = "kms-password-list")]
    pub(crate) kms_password_list: String,

    /// node list looks like localhost:40000:node0:k8s_cluster1:40000,localhost:40001:node1:k8s_cluster2:40000
    /// last slice is optional, none means not k8s env.
    #[clap(long = "nodelist")]
    pub(crate) node_list: String,

    /// log level
    #[clap(long = "log-level", default_value = "info")]
    pub(crate) log_level: String,
}

/// admin set by args
/// kms password set by args
/// grpc ports start from 50000
/// node network listen port is 40000
/// is stdout is true
pub fn execute_create_k8s(opts: CreateK8sOpts) -> Result<(), Error> {
    // init chain
    execute_init_chain(InitChainOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
    })
    .unwrap();

    // init chain config
    execute_init_chain_config(InitChainConfigOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        timestamp: opts.timestamp,
        prevhash: opts.prevhash.clone(),
        version: opts.version,
        chain_id: opts.chain_id.clone(),
        block_interval: opts.block_interval,
        block_limit: opts.block_limit,
        network_image: opts.network_image.clone(),
        network_tag: opts.network_tag.clone(),
        consensus_image: opts.consensus_image.clone(),
        consensus_tag: opts.consensus_tag.clone(),
        executor_image: opts.executor_image.clone(),
        executor_tag: opts.executor_tag.clone(),
        storage_image: opts.storage_image.clone(),
        storage_tag: opts.storage_tag.clone(),
        controller_image: opts.controller_image.clone(),
        controller_tag: opts.controller_tag.clone(),
        kms_image: opts.kms_image.clone(),
        kms_tag: opts.kms_tag.clone(),
    })
    .unwrap();

    // set admin
    execute_set_admin(SetAdminOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        admin: opts.admin.clone(),
    })
    .unwrap();

    // parse kms password list and node list
    let kms_password_list: Vec<&str> = opts.kms_password_list.split(',').collect();
    let node_list_str: Vec<&str> = opts.node_list.split(',').collect();
    let node_list: Vec<NodeNetworkAddress> = node_list_str
        .iter()
        .map(|node| {
            let node_network_info: Vec<&str> = node.split(':').collect();
            let cluster = if node_network_info.len() == 3 {
                rand_string()
            } else {
                node_network_info[3].to_string()
            };
            NodeNetworkAddressBuilder::new()
                .host(node_network_info[0].to_string())
                .port(node_network_info[1].parse::<u16>().unwrap())
                .domain(node_network_info[2].to_string())
                .cluster(cluster)
                .build()
        })
        .collect();

    if node_list.len() != kms_password_list.len() {
        return Err(Error::ListLenNotMatch);
    }

    // gen validator addr and append validator
    let mut node_accounts = Vec::new();
    for kms_password in kms_password_list.iter() {
        let (key_id, addr, validator_addr) = execute_new_account(NewAccountOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            kms_password: kms_password.to_string(),
        })
        .unwrap();
        execute_append_validator(AppendValidatorOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            validator: validator_addr.clone(),
        })
        .unwrap();
        node_accounts.push((key_id, addr));
    }

    // set node list
    execute_set_nodelist(SetNodeListOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        node_list: opts.node_list.clone(),
    })
    .unwrap();

    let is_tls = opts.network_image == NETWORK_TLS;

    // if network is tls
    // gen ca and gen cert for each node
    if is_tls {
        execute_create_ca(CreateCAOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
        })
        .unwrap();
        for node in node_list.iter() {
            let domain = node.domain.to_string();
            execute_create_csr(CreateCSROpts {
                chain_name: opts.chain_name.clone(),
                config_dir: opts.config_dir.clone(),
                domain: domain.clone(),
            })
            .unwrap();
            execute_sign_csr(SignCSROpts {
                chain_name: opts.chain_name.clone(),
                config_dir: opts.config_dir.clone(),
                domain: domain.clone(),
            })
            .unwrap();
        }
    }

    execute_set_stage(SetStageOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        stage: "finalize".to_string(),
    })
    .unwrap();

    // reload chainconfig
    let chain_config_file = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CHAIN_CONFIG_FILE
    );

    let chain_config = read_chain_config(&chain_config_file).unwrap();

    // init node and update node
    for (i, node) in chain_config.node_network_address_list.iter().enumerate() {
        let network_port = 50000;
        let domain = node.domain.to_string();
        let network_listen_port = 40000;
        let node_account = node_accounts[i].clone();
        let kms_password = kms_password_list[i].to_string();

        execute_init_node(InitNodeOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            domain: domain.clone(),
            network_port,
            consensus_port: network_port + 1,
            executor_port: network_port + 2,
            storage_port: network_port + 3,
            controller_port: network_port + 4,
            kms_port: network_port + 5,
            network_listen_port,
            kms_password,
            key_id: node_account.0,
            log_level: opts.log_level.clone(),
            account: node_account.1,
            package_limit: 30000,
        })
        .unwrap();

        execute_update_node(UpdateNodeOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            domain: domain.clone(),
            is_stdout: true,
            config_name: "config.toml".to_string(),
            is_old: false,
        })
        .unwrap();
    }

    Ok(())
}

#[derive(Parser, Debug, Clone)]
pub struct AppendK8sOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    config_dir: String,
    /// log level
    #[clap(long = "log-level", default_value = "info")]
    log_level: String,
    /// kms db password
    #[clap(long = "kms-password")]
    pub(crate) kms_password: String,
    /// node network address looks like localhost:40002:node2:k8s_cluster1
    /// last slice is optional, none means not k8s env.
    #[clap(long = "node")]
    pub(crate) node: String,
}

/// append a new node into chain
pub fn execute_append_k8s(opts: AppendK8sOpts) -> Result<(), Error> {
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CHAIN_CONFIG_FILE
    );
    let chain_config = read_chain_config(&file_name).unwrap();
    let is_tls = find_micro_service(&chain_config, NETWORK_TLS);

    // create account for new node
    let (key_id, addr, _) = execute_new_account(NewAccountOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        kms_password: opts.kms_password.clone(),
    })
    .unwrap();

    // parse node network info
    let node_network_info: Vec<&str> = opts.node.split(':').collect();
    let cluster = if node_network_info.len() == 3 {
        rand_string()
    } else {
        node_network_info[3].to_string()
    };

    let new_node = NodeNetworkAddressBuilder::new()
        .host(node_network_info[0].to_string())
        .port(node_network_info[1].parse::<u16>().unwrap())
        .domain(node_network_info[2].to_string())
        .cluster(cluster)
        .build();

    // append node
    execute_append_node(AppendNodeOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        node: opts.node.clone(),
    })
    .unwrap();

    // if network is tls
    // gen cert for new node
    if is_tls {
        let domain = new_node.domain.clone();
        execute_create_csr(CreateCSROpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            domain: domain.clone(),
        })
        .unwrap();
        execute_sign_csr(SignCSROpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            domain,
        })
        .unwrap();
    }

    // update old nodes
    // chain_config load befor append node, so it only contains old nodes
    for node in chain_config.node_network_address_list {
        let domain = node.domain.clone();

        execute_update_node(UpdateNodeOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            domain: domain.clone(),
            is_stdout: true,
            config_name: "config.toml".to_string(),
            is_old: true,
        })
        .unwrap();
    }

    // new node need init and update
    let network_port = 50000;
    let network_listen_port = 40000;
    let domain = new_node.domain;

    execute_init_node(InitNodeOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        domain: domain.clone(),
        network_port,
        consensus_port: network_port + 1,
        executor_port: network_port + 2,
        storage_port: network_port + 3,
        controller_port: network_port + 4,
        kms_port: network_port + 5,
        network_listen_port,
        kms_password: opts.kms_password.clone(),
        key_id,
        log_level: opts.log_level.clone(),
        account: addr,
        package_limit: 30000,
    })
    .unwrap();

    execute_update_node(UpdateNodeOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        domain,
        is_stdout: true,
        config_name: "config.toml".to_string(),
        is_old: false,
    })
    .unwrap();

    Ok(())
}

#[derive(Parser, Debug, Clone)]
pub struct DeleteK8sOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    config_dir: String,
    /// domain of node that want to delete
    #[clap(long = "domain")]
    pub(crate) domain: String,
}

pub fn execute_delete_k8s(opts: DeleteK8sOpts) -> Result<(), Error> {
    // delete node before load chain config
    execute_delete_node(DeleteNodeOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        domain: opts.domain.clone(),
        config_name: "config.toml".to_string(),
    })
    .unwrap();
    delete_node_folders(&opts.config_dir, &opts.chain_name, &opts.domain);

    // load chain config after delete node
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CHAIN_CONFIG_FILE
    );
    let chain_config = read_chain_config(&file_name).unwrap();

    // update reserve nodes
    for node in chain_config.node_network_address_list {
        let domain = node.domain.clone();

        execute_update_node(UpdateNodeOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            domain: domain.clone(),
            is_stdout: true,
            config_name: "config.toml".to_string(),
            is_old: true,
        })
        .unwrap();
    }

    Ok(())
}

#[cfg(test)]
mod k8s_test {
    use super::*;
    use crate::util::rand_string;

    #[test]
    fn k8s_test() {
        let name = rand_string();
        let name1 = rand_string();
        execute_create_k8s(CreateK8sOpts {
            chain_name: name.clone(),
            config_dir: "/tmp".to_string(),
            timestamp: 0,
            prevhash: "0x0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            version: 0,
            chain_id: "".to_string(),
            block_interval: 3,
            block_limit: 100,
            network_image: "network_tls".to_string(),
            network_tag: "latest".to_string(),
            consensus_image: "consensus_bft".to_string(),
            consensus_tag: "latest".to_string(),
            executor_image: "executor_evm".to_string(),
            executor_tag: "latest".to_string(),
            storage_image: "storage_rocksdb".to_string(),
            storage_tag: "latest".to_string(),
            controller_image: "controller".to_string(),
            controller_tag: "latest".to_string(),
            kms_image: "kms_sm".to_string(),
            kms_tag: "latest".to_string(),
            admin: "a81a6d5ebf5bb612dd52b37f743d2eb7a90807f7".to_string(),
            kms_password_list: "123,123".to_string(),
            node_list: "localhost:40000:node0:k8s:40000,localhost:40001:node1:k8s:40000"
                .to_string(),
            log_level: "info".to_string(),
        })
        .unwrap();

        execute_create_k8s(CreateK8sOpts {
            chain_name: name1,
            config_dir: "/tmp".to_string(),
            timestamp: 0,
            prevhash: "0x0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            version: 0,
            chain_id: "".to_string(),
            block_interval: 3,
            block_limit: 100,
            network_image: "network_p2p".to_string(),
            network_tag: "latest".to_string(),
            consensus_image: "consensus_raft".to_string(),
            consensus_tag: "latest".to_string(),
            executor_image: "executor_evm".to_string(),
            executor_tag: "latest".to_string(),
            storage_image: "storage_rocksdb".to_string(),
            storage_tag: "latest".to_string(),
            controller_image: "controller".to_string(),
            controller_tag: "latest".to_string(),
            kms_image: "kms_eth".to_string(),
            kms_tag: "latest".to_string(),
            admin: "a81a6d5ebf5bb612dd52b37f743d2eb7a90807f7".to_string(),
            kms_password_list: "123,123".to_string(),
            node_list: "localhost:40000:node0:k8s:40000,localhost:40001:node1:k8s:40000"
                .to_string(),
            log_level: "info".to_string(),
        })
        .unwrap();

        execute_append_k8s(AppendK8sOpts {
            chain_name: name.clone(),
            config_dir: "/tmp".to_string(),
            log_level: "info".to_string(),
            kms_password: "123".to_string(),
            node: "localhost:40002:node2:k8s:40000".to_string(),
        })
        .unwrap();

        execute_delete_k8s(DeleteK8sOpts {
            chain_name: name,
            config_dir: "/tmp".to_string(),
            domain: "node2".to_string(),
        })
        .unwrap();
    }
}
