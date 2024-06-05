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
use crate::constant::CHAIN_CONFIG_FILE;
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
use crate::util::read_chain_config;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;

/// A subcommand for run
#[derive(Parser, Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct CreateOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub config_dir: String,
    /// set genesis timestamp
    #[clap(long = "timestamp", default_value = "0")]
    pub timestamp: u64,
    /// set genesis prevhash
    #[clap(
        long = "prevhash",
        default_value = "0x0000000000000000000000000000000000000000000000000000000000000000"
    )]
    pub prevhash: String,
    /// set system config version
    #[clap(long = "version", default_value = "0")]
    pub version: u32,
    /// set system config chain_id
    #[clap(long = "chain_id", default_value = "")]
    pub chain_id: String,
    /// set system config block_interval
    #[clap(long = "block_interval", default_value = "3")]
    pub block_interval: u32,
    /// set system config block_limit
    #[clap(long = "block_limit", default_value = "100")]
    pub block_limit: u64,
    /// set one block contains quota limit, default 1073741824
    #[clap(long = "quota-limit", default_value = "1073741824")]
    pub quota_limit: u64,
    /// set network micro service image name (network_zenoh)
    #[clap(long = "network_image", default_value = "network_zenoh")]
    pub network_image: String,
    /// set network micro service image tag
    #[clap(long = "network_tag", default_value = "latest")]
    pub network_tag: String,
    /// set consensus micro service image name (consensus_raft/consensus_overlord)
    #[clap(long = "consensus_image", default_value = "consensus_overlord")]
    pub consensus_image: String,
    /// set consensus micro service image tag
    #[clap(long = "consensus_tag", default_value = "latest")]
    pub consensus_tag: String,
    /// set executor micro service image name (executor_evm)
    #[clap(long = "executor_image", default_value = "executor_evm")]
    pub executor_image: String,
    /// set executor micro service image tag
    #[clap(long = "executor_tag", default_value = "latest")]
    pub executor_tag: String,
    /// set storage micro service image name (storage_opendal)
    #[clap(long = "storage_image", default_value = "storage_opendal")]
    pub storage_image: String,
    /// set storage micro service image tag
    #[clap(long = "storage_tag", default_value = "latest")]
    pub storage_tag: String,
    /// set controller micro service image name (controller_hsm)
    #[clap(long = "controller_image", default_value = "controller_hsm")]
    pub controller_image: String,
    /// set controller micro service image tag
    #[clap(long = "controller_tag", default_value = "latest")]
    pub controller_tag: String,

    /// set admin
    #[clap(long = "admin")]
    pub admin: String,

    /// node list looks like localhost:40000:node0:k8s_cluster_name_1:namespace_1,localhost:40001:node1:k8s_cluster_name_2:namespace_2
    /// for each node network address:
    /// k8s_cluster_name is optional, none means not k8s env.
    /// namespace is optional, none means default namespace.
    #[clap(long = "nodelist")]
    pub node_list: String,

    /// log level
    #[clap(long = "log-level", default_value = "info")]
    pub log_level: String,

    /// log file path
    #[clap(long = "log-file-path")]
    pub log_file_path: Option<String>,

    /// jaeger agent endpoint
    #[clap(long = "jaeger-agent-endpoint")]
    pub jaeger_agent_endpoint: Option<String>,

    /// is chain in danger mode
    #[clap(long = "is-danger")]
    pub is_danger: bool,
    /// enable tx persistence
    #[clap(long = "enable-tx-persistence")]
    pub enable_tx_persistence: bool,

    /// disable metrics
    #[clap(long = "disable-metrics")]
    pub disable_metrics: bool,

    /// cloud_storage.access_key_id
    #[clap(long = "access-key-id", default_value = "")]
    pub access_key_id: String,
    /// cloud_storage.secret_access_key
    #[clap(long = "secret-access-key", default_value = "")]
    pub secret_access_key: String,
    /// cloud_storage.endpoint
    #[clap(long = "s3-endpoint", default_value = "")]
    pub s3_endpoint: String,
    /// cloud_storage.bucket
    #[clap(long = "s3-bucket", default_value = "")]
    pub s3_bucket: String,
    /// cloud_storage.service_type: s3/oss(aliyun)/obs(huawei)/cos(tencent)/azblob(azure)
    #[clap(long = "service-type", default_value = "")]
    pub service_type: String,
    /// cloud_storage.root
    #[clap(long = "s3-root", default_value = "")]
    pub s3_root: String,
    /// cloud_storage.region
    #[clap(long = "s3-region", default_value = "")]
    pub s3_region: String,
    /// exporter.base_path
    #[clap(long = "exporter-path", default_value = "")]
    pub exporter_path: String,
}

impl Default for CreateOpts {
    fn default() -> Self {
        Self {
            chain_name: "test-chain".to_string(),
            config_dir: ".".to_string(),
            timestamp: 0,
            prevhash: "0x0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            version: 0,
            chain_id: Default::default(),
            block_interval: 3,
            block_limit: 100,
            quota_limit: 1073741824,
            network_image: "network_zenoh".to_string(),
            network_tag: "latest".to_string(),
            consensus_image: "consensus_overlord".to_string(),
            consensus_tag: "latest".to_string(),
            executor_image: "executor_evm".to_string(),
            executor_tag: "latest".to_string(),
            storage_image: "storage_opendal".to_string(),
            storage_tag: "latest".to_string(),
            controller_image: "controller_hsm".to_string(),
            controller_tag: "latest".to_string(),
            admin: Default::default(),
            node_list: Default::default(),
            log_level: "info".to_string(),
            log_file_path: Default::default(),
            jaeger_agent_endpoint: Default::default(),
            is_danger: Default::default(),
            enable_tx_persistence: Default::default(),
            disable_metrics: Default::default(),
            access_key_id: "".to_string(),
            secret_access_key: "".to_string(),
            s3_endpoint: "".to_string(),
            s3_bucket: "".to_string(),
            service_type: "".to_string(),
            s3_root: "".to_string(),
            s3_region: "".to_string(),
            exporter_path: "".to_string(),
        }
    }
}

/// admin set by args
/// grpc ports start from 50000
/// node network listen port is 40000
/// is stdout is true
pub fn execute_create(opts: CreateOpts) -> Result<(), Error> {
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
        quota_limit: opts.quota_limit,
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
    })
    .unwrap();

    // set admin
    execute_set_admin(SetAdminOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        admin: opts.admin.clone(),
    })
    .unwrap();

    // parse node list
    let node_list_str: Vec<&str> = opts.node_list.split(',').collect();
    let node_list: Vec<NodeNetworkAddress> = node_list_str
        .iter()
        .map(|node| {
            let node_network_info: Vec<&str> = node.split(':').collect();
            if node_network_info.len() == 3 {
                NodeNetworkAddressBuilder::default()
                    .host(node_network_info[0].to_string())
                    .port(node_network_info[1].parse::<u16>().unwrap())
                    .domain(node_network_info[2].to_string())
                    .build()
            } else if node_network_info.len() == 4 {
                NodeNetworkAddressBuilder::default()
                    .host(node_network_info[0].to_string())
                    .port(node_network_info[1].parse::<u16>().unwrap())
                    .domain(node_network_info[2].to_string())
                    .cluster(node_network_info[3].to_string())
                    .build()
            } else if node_network_info.len() == 5 {
                NodeNetworkAddressBuilder::default()
                    .host(node_network_info[0].to_string())
                    .port(node_network_info[1].parse::<u16>().unwrap())
                    .domain(node_network_info[2].to_string())
                    .cluster(node_network_info[3].to_string())
                    .name_space(node_network_info[4].to_string())
                    .build()
            } else {
                panic!("invalid node network address format!")
            }
        })
        .collect();

    // gen validator addr and append validator
    let mut node_accounts = Vec::new();
    for _ in 0..node_list.len() {
        let (addr, validator_addr) = execute_new_account(NewAccountOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
        })
        .unwrap();
        execute_append_validator(AppendValidatorOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            validator: validator_addr.clone(),
        })
        .unwrap();
        node_accounts.push(addr);
    }

    // set node list
    execute_set_nodelist(SetNodeListOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        node_list: opts.node_list.clone(),
    })
    .unwrap();

    // gen ca and gen cert for each node
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

    let chain_config = read_chain_config(chain_config_file).unwrap();

    // init node and update node
    for (i, node) in chain_config.node_network_address_list.iter().enumerate() {
        // for k8s node offset is 0
        // for none k8s node offset used to avoid port conflict
        let offset = if node.cluster.is_empty() {
            (node.port - 40000) * 100
        } else {
            0
        };
        let network_port = 50000 + offset;
        let network_metrics_port = 60000 + offset;
        let domain = node.domain.to_string();
        let node_account = node_accounts[i].clone();

        execute_init_node(InitNodeOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            domain: domain.clone(),
            network_port,
            consensus_port: network_port + 1,
            executor_port: network_port + 2,
            storage_port: network_port + 3,
            controller_port: network_port + 4,
            log_level: opts.log_level.clone(),
            log_file_path: opts.log_file_path.clone(),
            jaeger_agent_endpoint: opts.jaeger_agent_endpoint.clone(),
            account: node_account,
            network_metrics_port,
            consensus_metrics_port: network_metrics_port + 1,
            executor_metrics_port: network_metrics_port + 2,
            storage_metrics_port: network_metrics_port + 3,
            controller_metrics_port: network_metrics_port + 4,
            disable_metrics: opts.disable_metrics,
            is_danger: opts.is_danger,
            enable_tx_persistence: opts.enable_tx_persistence,
            access_key_id: opts.access_key_id.clone(),
            secret_access_key: opts.secret_access_key.clone(),
            s3_endpoint: opts.s3_endpoint.clone(),
            s3_bucket: opts.s3_bucket.clone(),
            service_type: opts.service_type.clone(),
            s3_root: opts.s3_root.clone(),
            s3_region: opts.s3_region.clone(),
            exporter_path: opts.exporter_path.clone(),
        })
        .unwrap();

        execute_update_node(UpdateNodeOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            domain: domain.clone(),
            config_name: "config.toml".to_string(),
        })
        .unwrap();
    }

    Ok(())
}

#[derive(Parser, Debug, Clone)]
pub struct AppendOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub config_dir: String,
    /// log level
    #[clap(long = "log-level", default_value = "info")]
    pub log_level: String,
    /// log file path
    #[clap(long = "log-file-path")]
    pub log_file_path: Option<String>,
    /// jaeger agent endpoint
    #[clap(long = "jaeger-agent-endpoint")]
    pub jaeger_agent_endpoint: Option<String>,
    /// node network address looks like localhost:40002:node2:k8s_cluster_name:namespace
    /// k8s_cluster_name is optional, none means not k8s env.
    /// namespace is optional, none means default namespace.
    #[clap(long = "node")]
    pub node: String,
    /// is chain in danger mode
    #[clap(long = "is-danger")]
    pub is_danger: bool,
    /// enable tx persistence
    #[clap(long = "enable-tx-persistence")]
    pub enable_tx_persistence: bool,
    /// disable metrics
    #[clap(long = "disable-metrics")]
    pub disable_metrics: bool,
    /// cloud_storage.access_key_id
    #[clap(long = "access-key-id", default_value = "")]
    pub access_key_id: String,
    /// cloud_storage.secret_access_key
    #[clap(long = "secret-access-key", default_value = "")]
    pub secret_access_key: String,
    /// cloud_storage.endpoint
    #[clap(long = "s3-endpoint", default_value = "")]
    pub s3_endpoint: String,
    /// cloud_storage.bucket
    #[clap(long = "s3-bucket", default_value = "")]
    pub s3_bucket: String,
    /// cloud_storage.service_type: s3/oss(aliyun)/obs(huawei)/cos(tencent)/azblob(azure)
    #[clap(long = "service-type", default_value = "")]
    pub service_type: String,
    /// cloud_storage.root
    #[clap(long = "s3-root", default_value = "")]
    pub s3_root: String,
    /// cloud_storage.region
    #[clap(long = "s3-region", default_value = "")]
    pub s3_region: String,
    /// exporter.base_path
    #[clap(long = "exporter-path", default_value = "")]
    pub exporter_path: String,
}

/// append a new node into chain
pub fn execute_append(opts: AppendOpts) -> Result<(), Error> {
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CHAIN_CONFIG_FILE
    );
    let chain_config = read_chain_config(file_name).unwrap();

    // create account for new node
    let (addr, _) = execute_new_account(NewAccountOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
    })
    .unwrap();

    // parse node network info
    let node_network_info: Vec<&str> = opts.node.split(':').collect();
    let new_node = if node_network_info.len() == 3 {
        NodeNetworkAddressBuilder::default()
            .host(node_network_info[0].to_string())
            .port(node_network_info[1].parse::<u16>().unwrap())
            .domain(node_network_info[2].to_string())
            .build()
    } else if node_network_info.len() == 4 {
        NodeNetworkAddressBuilder::default()
            .host(node_network_info[0].to_string())
            .port(node_network_info[1].parse::<u16>().unwrap())
            .domain(node_network_info[2].to_string())
            .cluster(node_network_info[3].to_string())
            .build()
    } else if node_network_info.len() == 5 {
        NodeNetworkAddressBuilder::default()
            .host(node_network_info[0].to_string())
            .port(node_network_info[1].parse::<u16>().unwrap())
            .domain(node_network_info[2].to_string())
            .cluster(node_network_info[3].to_string())
            .name_space(node_network_info[4].to_string())
            .build()
    } else {
        panic!("invalid node network address format!")
    };

    // append node
    execute_append_node(AppendNodeOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        node: opts.node.clone(),
    })
    .unwrap();

    // gen cert for new node
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

    // update old nodes
    // chain_config load before append node, so it only contains old nodes
    for node in chain_config.node_network_address_list {
        let domain = node.domain.clone();

        // chain_config modified, update for old nodes
        let from = format!(
            "{}/{}/{}",
            &opts.config_dir, &opts.chain_name, CHAIN_CONFIG_FILE
        );
        let node_dir = format!("{}/{}-{}", &opts.config_dir, &opts.chain_name, &domain);
        let to = format!("{}/{}", &node_dir, CHAIN_CONFIG_FILE);
        fs::copy(from, to).unwrap();

        execute_update_node(UpdateNodeOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            domain: domain.clone(),
            config_name: "config.toml".to_string(),
        })
        .unwrap();
    }

    // new node need init and update
    // for k8s node offset is 0
    // for none k8s node offset used to avoid port conflict
    let offset = if new_node.cluster.is_empty() {
        (new_node.port - 40000) * 100
    } else {
        0
    };
    let network_port = 50000 + offset;
    let network_metrics_port = 60000 + offset;
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
        log_level: opts.log_level,
        log_file_path: opts.log_file_path,
        jaeger_agent_endpoint: opts.jaeger_agent_endpoint,
        account: addr,
        network_metrics_port,
        consensus_metrics_port: network_metrics_port + 1,
        executor_metrics_port: network_metrics_port + 2,
        storage_metrics_port: network_metrics_port + 3,
        controller_metrics_port: network_metrics_port + 4,
        disable_metrics: opts.disable_metrics,
        is_danger: opts.is_danger,
        enable_tx_persistence: opts.enable_tx_persistence,
        access_key_id: opts.access_key_id.clone(),
        secret_access_key: opts.secret_access_key.clone(),
        s3_endpoint: opts.s3_endpoint.clone(),
        s3_bucket: opts.s3_bucket.clone(),
        service_type: opts.service_type.clone(),
        s3_root: opts.s3_root.clone(),
        s3_region: opts.s3_region.clone(),
        exporter_path: opts.exporter_path.clone(),
    })
    .unwrap();

    execute_update_node(UpdateNodeOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        domain,
        config_name: "config.toml".to_string(),
    })
    .unwrap();

    Ok(())
}

#[derive(Parser, Debug, Clone)]
pub struct DeleteOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub config_dir: String,
    /// domain of node that want to delete
    #[clap(long = "domain")]
    pub domain: String,
}

pub fn execute_delete(opts: DeleteOpts) -> Result<(), Error> {
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
    let chain_config = read_chain_config(file_name).unwrap();

    // update reserve nodes
    for node in chain_config.node_network_address_list {
        let domain = node.domain.clone();

        // chain_config modified, update for old nodes
        let from = format!(
            "{}/{}/{}",
            &opts.config_dir, &opts.chain_name, CHAIN_CONFIG_FILE
        );
        let node_dir = format!("{}/{}-{}", &opts.config_dir, &opts.chain_name, &domain);
        let to = format!("{}/{}", &node_dir, CHAIN_CONFIG_FILE);
        fs::copy(from, to).unwrap();

        execute_update_node(UpdateNodeOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            domain: domain.clone(),
            config_name: "config.toml".to_string(),
        })
        .unwrap();
    }

    Ok(())
}

#[cfg(test)]
mod cmd_test {
    use super::*;
    use crate::delete_chain::{execute_delete_chain, DeleteChainOpts};

    #[test]
    fn cmd_test() {
        let name = "test-chain".to_string();
        let name1 = "test-chain-1".to_string();
        execute_create(CreateOpts {
            chain_name: name.clone(),
            config_dir: "/tmp".to_string(),
            timestamp: 0,
            prevhash: "0x0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            version: 0,
            chain_id: "".to_string(),
            block_interval: 3,
            block_limit: 100,
            quota_limit: 1073741824,
            network_image: "network_zenoh".to_string(),
            network_tag: "latest".to_string(),
            consensus_image: "consensus_overlord".to_string(),
            consensus_tag: "latest".to_string(),
            executor_image: "executor_evm".to_string(),
            executor_tag: "latest".to_string(),
            storage_image: "storage_opendal".to_string(),
            storage_tag: "latest".to_string(),
            controller_image: "controller_hsm".to_string(),
            controller_tag: "latest".to_string(),
            admin: "a81a6d5ebf5bb612dd52b37f743d2eb7a90807f7".to_string(),
            node_list: "localhost:40000:node0:k8s,localh
            ost:40001:node1:k8s,localhost:40002:node2:k8s:cita,rivtower.com:40003:node3,192.168.160.20:40004:node4"
                .to_string(),
            log_level: "info".to_string(),
            log_file_path: None,
            jaeger_agent_endpoint: None,
            is_danger: false,
            enable_tx_persistence: false,
            disable_metrics: false,
            access_key_id: "".to_string(),
            secret_access_key: "".to_string(),
            s3_endpoint: "".to_string(),
            s3_bucket: "".to_string(),
            service_type: "".to_string(),
            s3_root: "".to_string(),
            s3_region: "".to_string(),
            exporter_path: "".to_string(),
        })
        .unwrap();

        execute_create(CreateOpts {
            chain_name: name1.clone(),
            config_dir: "/tmp".to_string(),
            timestamp: 0,
            prevhash: "0x0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
            version: 0,
            chain_id: "".to_string(),
            block_interval: 3,
            block_limit: 100,
            quota_limit: 1073741824,
            network_image: "network_zenoh".to_string(),
            network_tag: "latest".to_string(),
            consensus_image: "consensus_raft".to_string(),
            consensus_tag: "latest".to_string(),
            executor_image: "executor_evm".to_string(),
            executor_tag: "latest".to_string(),
            storage_image: "storage_opendal".to_string(),
            storage_tag: "latest".to_string(),
            controller_image: "controller_hsm".to_string(),
            controller_tag: "latest".to_string(),
            admin: "a81a6d5ebf5bb612dd52b37f743d2eb7a90807f7".to_string(),
            node_list: "localhost:40000:node0:k8s,localhost:40001:node1:k8s".to_string(),
            log_level: "info".to_string(),
            log_file_path: None,
            jaeger_agent_endpoint: None,
            is_danger: false,
            enable_tx_persistence: false,
            disable_metrics: false,
            access_key_id: "".to_string(),
            secret_access_key: "".to_string(),
            s3_endpoint: "".to_string(),
            s3_bucket: "".to_string(),
            service_type: "".to_string(),
            s3_root: "".to_string(),
            s3_region: "".to_string(),
            exporter_path: "".to_string(),
        })
        .unwrap();

        execute_append(AppendOpts {
            chain_name: name1.clone(),
            config_dir: "/tmp".to_string(),
            log_level: "info".to_string(),
            node: "localhost:40002:node2:k8s".to_string(),
            log_file_path: None,
            jaeger_agent_endpoint: None,
            is_danger: false,
            enable_tx_persistence: false,
            disable_metrics: false,
            access_key_id: "".to_string(),
            secret_access_key: "".to_string(),
            s3_endpoint: "".to_string(),
            s3_bucket: "".to_string(),
            service_type: "".to_string(),
            s3_root: "".to_string(),
            s3_region: "".to_string(),
            exporter_path: "".to_string(),
        })
        .unwrap();

        execute_delete(DeleteOpts {
            chain_name: name1.clone(),
            config_dir: "/tmp".to_string(),
            domain: "node2".to_string(),
        })
        .unwrap();

        // clean
        let _ = execute_delete_chain(DeleteChainOpts {
            chain_name: name.clone(),
            config_dir: "/tmp".to_string(),
        });
        let _ = execute_delete_chain(DeleteChainOpts {
            chain_name: name1.clone(),
            config_dir: "/tmp".to_string(),
        });
    }
}
