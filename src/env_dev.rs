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
use crate::constant::{CHAIN_CONFIG_FILE, CONSENSUS_RAFT, CRYPTO_ETH};
use crate::create_ca::{execute_create_ca, CreateCAOpts};
use crate::create_csr::{execute_create_csr, CreateCSROpts};
use crate::delete_node::{delete_node_folders, execute_delete_node, DeleteNodeOpts};
use crate::error::Error;
use crate::init_chain::{execute_init_chain, InitChainOpts};
use crate::init_chain_config::{execute_init_chain_config, InitChainConfigOpts};
use crate::init_node::{execute_init_node, InitNodeOpts};
use crate::new_account::{execute_new_account, NewAccountOpts};
use crate::set_admin::{execute_set_admin, SetAdminOpts};
use crate::set_stage::{execute_set_stage, SetStageOpts};
use crate::sign_csr::{execute_sign_csr, SignCSROpts};
use crate::update_node::{execute_update_node, UpdateNodeOpts};
use crate::util::read_chain_config;
use clap::Parser;
use std::fs;

/// A subcommand for run
#[derive(Parser, Debug, Clone)]
pub struct CreateDevOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    config_dir: String,
    /// set initial node number
    #[clap(long = "peers-count", default_value = "4")]
    peers_count: u16,
    /// log level
    #[clap(long = "log-level", default_value = "info")]
    log_level: String,
    /// log file path
    #[clap(long = "log-file-path", default_value = "./logs")]
    log_file_path: Option<String>,
    /// jaeger agent endpoint
    #[clap(long = "jaeger-agent-endpoint")]
    jaeger_agent_endpoint: Option<String>,
    /// is consensus raft
    #[clap(long = "is-raft")]
    is_raft: bool,
    /// is crypto eth
    #[clap(long = "is-eth")]
    is_eth: bool,
    /// is chain in danger mode
    #[clap(long = "is-danger")]
    is_danger: bool,
    /// disable metrics
    #[clap(long = "disable-metrics")]
    pub(crate) disable_metrics: bool,
}

/// node network ip is 127.0.0.1
/// node network port is 40000 + i
/// node domain is i
/// grpc ports start from 50000 + i*1000
/// metrics ports start from 60000 + i*100
/// node network listen port is 40000 + i
/// is stdout is false
pub fn execute_create_dev(opts: CreateDevOpts) -> Result<(), Error> {
    let peers_count = opts.peers_count as usize;

    // init chain
    execute_init_chain(InitChainOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
    })
    .unwrap();

    // init chain config
    let mut init_chain_config_opts = InitChainConfigOpts::parse_from(vec![""]);
    init_chain_config_opts.chain_name = opts.chain_name.clone();
    init_chain_config_opts.config_dir = opts.config_dir.clone();
    // is_raft will override overlord
    if opts.is_raft {
        init_chain_config_opts.consensus_image = CONSENSUS_RAFT.to_string();
    }
    if opts.is_eth {
        init_chain_config_opts.crypto_image = CRYPTO_ETH.to_string();
    }
    execute_init_chain_config(init_chain_config_opts).unwrap();

    // gen admin addr and set admin
    let (admin_addr, _) = execute_new_account(NewAccountOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
    })
    .unwrap();
    execute_set_admin(SetAdminOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        admin: admin_addr,
    })
    .unwrap();

    // gen validator addr and append validator
    let mut node_accounts = Vec::new();
    for _ in 0..peers_count {
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

    // append node
    for i in 0..peers_count {
        let node = format!("127.0.0.1:{}:{}", 40000 + i, i);
        execute_append_node(AppendNodeOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            node,
        })
        .unwrap();
    }

    // gen ca and gen cert for each node
    execute_create_ca(CreateCAOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
    })
    .unwrap();
    for i in 0..peers_count {
        let domain = format!("{i}");
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

    #[allow(clippy::needless_range_loop)]
    for i in 0..peers_count {
        let network_port = (50000 + i * 1000) as u16;
        let network_metrics_port = (60000 + i * 100) as u16;
        let domain = format!("{i}");
        let listen_port = (40000 + i) as u16;
        let node_addr = node_accounts[i].clone();

        execute_init_node(InitNodeOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            domain: domain.clone(),
            network_port,
            consensus_port: network_port + 1,
            executor_port: network_port + 2,
            storage_port: network_port + 3,
            controller_port: network_port + 4,
            crypto_port: network_port + 5,
            network_listen_port: listen_port,
            log_level: opts.log_level.clone(),
            log_file_path: opts.log_file_path.clone(),
            jaeger_agent_endpoint: opts.jaeger_agent_endpoint.clone(),
            account: node_addr,
            network_metrics_port,
            consensus_metrics_port: network_metrics_port + 1,
            executor_metrics_port: network_metrics_port + 2,
            storage_metrics_port: network_metrics_port + 3,
            controller_metrics_port: network_metrics_port + 4,
            crypto_metrics_port: network_metrics_port + 5,
            disable_metrics: opts.disable_metrics,
            is_danger: opts.is_danger,
        })
        .unwrap();

        execute_update_node(UpdateNodeOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            domain,
            config_name: "config.toml".to_string(),
            is_dev: true,
        })
        .unwrap();
    }

    Ok(())
}

#[derive(Parser, Debug, Clone)]
pub struct AppendDevOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    config_dir: String,
    /// log level
    #[clap(long = "log-level", default_value = "info")]
    log_level: String,
    /// log file path
    #[clap(long = "log-file-path")]
    log_file_path: Option<String>,
    /// jaeger agent endpoint
    #[clap(long = "jaeger-agent-endpoint")]
    jaeger_agent_endpoint: Option<String>,
    /// is chain in danger mode
    #[clap(long = "is-danger")]
    is_danger: bool,
    /// disable metrics
    #[clap(long = "disable-metrics")]
    pub(crate) disable_metrics: bool,
}

/// append a new node into chain
pub fn execute_append_dev(opts: AppendDevOpts) -> Result<(), Error> {
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CHAIN_CONFIG_FILE
    );
    let chain_config = read_chain_config(file_name).unwrap();
    let peers_count = chain_config.node_network_address_list.len();
    let new_node_id = peers_count;

    // create account for new node
    let (addr, _) = execute_new_account(NewAccountOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
    })
    .unwrap();

    // append node
    let node = format!("127.0.0.1:{}:{}", 40000 + new_node_id, new_node_id);
    execute_append_node(AppendNodeOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        node,
    })
    .unwrap();

    // gen cert for new node
    let domain = format!("{new_node_id}");
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
    for i in 0..peers_count {
        let domain = format!("{i}");

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
            domain,
            config_name: "config.toml".to_string(),
            is_dev: true,
        })
        .unwrap();
    }

    // new node need init and update
    let network_port = (50000 + new_node_id * 1000) as u16;
    let network_metrics_port = (60000 + new_node_id * 100) as u16;
    let domain = format!("{new_node_id}");
    let listen_port = (40000 + new_node_id) as u16;

    execute_init_node(InitNodeOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        domain: domain.clone(),
        network_port,
        consensus_port: network_port + 1,
        executor_port: network_port + 2,
        storage_port: network_port + 3,
        controller_port: network_port + 4,
        crypto_port: network_port + 5,
        network_listen_port: listen_port,
        log_level: opts.log_level.clone(),
        log_file_path: opts.log_file_path.clone(),
        jaeger_agent_endpoint: opts.jaeger_agent_endpoint.clone(),
        account: addr,
        network_metrics_port,
        consensus_metrics_port: network_metrics_port + 1,
        executor_metrics_port: network_metrics_port + 2,
        storage_metrics_port: network_metrics_port + 3,
        controller_metrics_port: network_metrics_port + 4,
        crypto_metrics_port: network_metrics_port + 5,
        disable_metrics: opts.disable_metrics,
        is_danger: opts.is_danger,
    })
    .unwrap();

    execute_update_node(UpdateNodeOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir,
        domain,
        config_name: "config.toml".to_string(),
        is_dev: true,
    })
    .unwrap();

    Ok(())
}

#[derive(Parser, Debug, Clone)]
pub struct DeleteDevOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    config_dir: String,
}

pub fn execute_delete_dev(opts: DeleteDevOpts) -> Result<(), Error> {
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CHAIN_CONFIG_FILE
    );
    let chain_config = read_chain_config(file_name).unwrap();
    let peers_count = chain_config.node_network_address_list.len();
    let delete_node_id = peers_count - 1;

    let domain = format!("{delete_node_id}");
    execute_delete_node(DeleteNodeOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        domain: domain.clone(),
        config_name: "config.toml".to_string(),
    })
    .unwrap();
    delete_node_folders(&opts.config_dir, &opts.chain_name, &domain);

    // update reserve nodes
    for i in 0..delete_node_id {
        let domain = format!("{i}");

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
            domain,
            config_name: "config.toml".to_string(),
            is_dev: true,
        })
        .unwrap();
    }

    Ok(())
}

#[cfg(test)]
mod dev_test {
    use super::*;
    use crate::util::rand_string;

    #[test]
    fn dev_test() {
        let name = rand_string();
        let name1 = rand_string();
        execute_create_dev(CreateDevOpts {
            chain_name: name.clone(),
            config_dir: ".tmp".to_string(),
            peers_count: 2,
            log_level: "info".to_string(),
            log_file_path: None,
            jaeger_agent_endpoint: None,
            is_raft: false,
            is_eth: false,
            is_danger: false,
            disable_metrics: false,
        })
        .unwrap();

        execute_create_dev(CreateDevOpts {
            chain_name: name1,
            config_dir: ".tmp".to_string(),
            peers_count: 2,
            log_level: "info".to_string(),
            log_file_path: None,
            jaeger_agent_endpoint: None,
            is_raft: true,
            is_eth: true,
            is_danger: false,
            disable_metrics: false,
        })
        .unwrap();

        execute_append_dev(AppendDevOpts {
            chain_name: name.clone(),
            config_dir: ".tmp".to_string(),
            log_level: "info".to_string(),
            log_file_path: None,
            jaeger_agent_endpoint: None,
            is_danger: false,
            disable_metrics: false,
        })
        .unwrap();

        execute_delete_dev(DeleteDevOpts {
            chain_name: name,
            config_dir: ".tmp".to_string(),
        })
        .unwrap();
    }
}
