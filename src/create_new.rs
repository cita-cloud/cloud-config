use crate::error::Error;
use crate::constant::{CONTROLLER, CONSENSUS_BFT, CONSENSUS_RAFT, NETWORK_P2P, NETWORK_TLS, EXECUTOR_EVM, KMS_SM, STORAGE_ROCKSDB, DEFAULT_VALUE};
use crate::init_chain::{execute_init_chain, InitChainOpts};
use clap::Clap;
use crate::init_chain_config::{execute_init_chain_config, InitChainConfigOpts};
use crate::new_account::{execute_new_account, NewAccountOpts};
use crate::set_admin::{execute_set_admin, SetAdminOpts};
use crate::set_validators::{execute_set_validators, SetValidatorsOpts};
use crate::set_nodelist::{execute_set_nodelist, SetNodeListOpts};
use crate::init_node::{execute_init_node, InitNodeOpts};
use crate::update_node::{execute_update_node, UpdateNodeOpts};
use crate::create_ca::{execute_create_ca, CreateCAOpts};
use crate::create_csr::{execute_create_csr, CreateCSROpts};
use crate::sign_csr::{execute_sign_csr, SignCSROpts};


/// A subcommand for run
#[derive(Clap, Debug, Clone)]
pub struct CreateLocalOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    config_dir: String,
    /// set genesis timestamp
    #[clap(long = "timestamp", default_value = "0")]
    timestamp: u64,
    /// set genesis prevhash
    #[clap(
    long = "prevhash",
    default_value = "0x0000000000000000000000000000000000000000000000000000000000000000"
    )]
    prevhash: String,
    /// set system config version
    #[clap(long = "version", default_value = "0")]
    version: u32,
    /// set system config chain_id
    #[clap(long = "chain_id", default_value = "")]
    chain_id: String,
    /// set system config block_interval
    #[clap(long = "block_interval", default_value = "3")]
    block_interval: u32,
    /// set system config block_limit
    #[clap(long = "block_limit", default_value = "100")]
    block_limit: u64,
    /// set network micro service image name (network_tls/network_p2p)
    #[clap(long = "network_image", default_value = "network_p2p")]
    network_image: String,
    /// set network micro service image tag
    #[clap(long = "network_tag", default_value = "latest")]
    network_tag: String,
    /// set consensus micro service image name (consensus_bft/consensus_raft)
    #[clap(long = "consensus_image", default_value = "consensus_raft")]
    consensus_image: String,
    /// set consensus micro service image tag
    #[clap(long = "consensus_tag", default_value = "latest")]
    consensus_tag: String,
    /// set executor micro service image name (executor_evm)
    #[clap(long = "executor_image", default_value = "executor_evm")]
    executor_image: String,
    /// set executor micro service image tag
    #[clap(long = "executor_tag", default_value = "latest")]
    executor_tag: String,
    /// set storage micro service image name (storage_rocksdb)
    #[clap(long = "storage_image", default_value = "storage_rocksdb")]
    storage_image: String,
    /// set storage micro service image tag
    #[clap(long = "storage_tag", default_value = "latest")]
    storage_tag: String,
    /// set controller micro service image name (controller)
    #[clap(long = "controller_image", default_value = "controller")]
    controller_image: String,
    /// set controller micro service image tag
    #[clap(long = "controller_tag", default_value = "latest")]
    controller_tag: String,
    /// set kms micro service image name (kms_eth/kms_sm)
    #[clap(long = "kms_image", default_value = "kms_sm")]
    kms_image: String,
    /// set kms micro service image tag
    #[clap(long = "kms_tag", default_value = "latest")]
    kms_tag: String,
    /// grpc port list, input "p1,p2,p3,p4", use default grpc port count from 50000 + 1000 * i
    /// use default must set peer_count or p2p_ports
    #[clap(long = "grpc-ports", default_value = "default")]
    grpc_ports: String,
    /// node list, input "localhost:40000:node0,localhost:40001:node1", use default port count from
    /// 127.0.0.1:40000 + 1 * i:nodei, use default must set peer_count or grpc_ports
    #[clap(long = "node-list", default_value = "default")]
    node_list: String,
    /// p2p listen port list, use default port count from 40000 + 1 * i, use default must set
    /// peer_count or grpc_ports or p2p_ports
    #[clap(long = "p2p-listen-ports", default_value = "default")]
    p2p_listen_ports: String,
    /// set initial node number, default "none" mean not use this must set grpc_ports or p2p_ports,
    /// if set peers_count, grpc_ports and p2p_ports, base on grpc_ports > p2p_ports > peers_count
    #[clap(long = "peers-count")]
    peers_count: Option<u16>,
    /// set config file name
    #[clap(long = "config-name", default_value = "config.toml")]
    config_name: String,
    /// kms db password
    #[clap(long = "kms-password", default_value = "123456")]
    kms_password: String,
    /// key id of account in kms db
    #[clap(long = "log-level", default_value = "info")]
    log_level: String,
    /// set one block contains tx limit, default 30000
    #[clap(long = "package-limit", default_value = "30000")]
    package_limit: u64,
}

pub fn execute_create(opts: CreateLocalOpts) -> Result<(), Error> {
    if opts.controller_image.as_str() != CONTROLLER {
        return Err(Error::ControllerNotExist);
    }
    if opts.consensus_image.as_str() != CONSENSUS_BFT && opts.consensus_image.as_str() != CONSENSUS_RAFT {
        return Err(Error::ConsensusNotExist);
    }
    if opts.network_image.as_str() != NETWORK_P2P && opts.network_image.as_str() != NETWORK_TLS {
        return Err(Error::NetworkNotExist);
    }
    if opts.executor_image.as_str() != EXECUTOR_EVM {
        return Err(Error::ExecutorNotExist);
    }
    if opts.kms_image.as_str() != KMS_SM {
        return Err(Error::KmsNotDefaultOrKmsSm);
    }
    if opts.storage_image.as_str() != STORAGE_ROCKSDB {
        return Err(Error::StorageNotExist);
    }
    let mut domains: Vec<String> = Vec::new();
    let mut ips: Vec<String> = Vec::new();
    let mut listen_ports: Vec<u16> = Vec::new();
    let mut network_ports: Vec<u16> = Vec::new();
    if opts.grpc_ports == DEFAULT_VALUE {
        if opts.node_list == DEFAULT_VALUE && opts.peers_count == None {
            return Err(Error::NodeCountNotExist);
        }
        if opts.node_list != DEFAULT_VALUE {
            let url: Vec<String> = opts.node_list.split(',').map(String::from).collect();
            for i in 0..listen_ports.len() {
                let pair: Vec<&str> = url[i].split(":").collect();
                domains.push(pair[2].clone().to_string());
                listen_ports.push(pair[1].parse().unwrap());
                network_ports.push((50000 + i * 1000) as u16);
                ips.push(pair[0].into());
            }

        } else {
            let peers_count = opts.peers_count.unwrap() as usize;
            for i in 0..peers_count {
                listen_ports.push((40000 + i) as u16);
                network_ports.push((50000 + i * 1000) as u16);
                domains.push(format!("node{}", i));
                ips.push("localhost".to_string());
            }

        }
    } else {
        if opts.node_list != DEFAULT_VALUE
            && opts.node_list.split(',').count() != opts.grpc_ports.split(',').count()
        {
            return Err(Error::P2pPortsParamNotValid);
        }
        let temp_ports: Vec<String> = opts.grpc_ports.split(',').map(String::from).collect();
        for item in &temp_ports {
            if item.parse::<u16>().is_err() {
                return Err(Error::GrpcPortsParamNotValid);
            }
            network_ports.push(item.parse().unwrap());
        }
        if opts.node_list == DEFAULT_VALUE {
            for i in 0..temp_ports.len() {
                listen_ports.push((40000 + i) as u16);
                domains.push(format!("node{}", i));
                ips.push("localhost".to_string());
            }
        } else {
            let node_list: Vec<String> = opts.node_list.split(',').map(String::from).collect();
            for i in 0..node_list.len() {
                let pair: Vec<&str> = node_list[i].split(":").collect();
                listen_ports.push(pair[1].parse().unwrap());
                domains.push(pair[2].parse().unwrap());
                ips.push(pair[0].to_string());
            }

        }

    }

    let mut node_list = String::from("");
    for i in 0..domains.len() - 1 {
        node_list = format!("{}{}", node_list, format!("{}:{}:{},", ips[i], listen_ports[i], domains[i]));
    }
    let last_index = domains.len() - 1;
    node_list = format!("{}{}", node_list, format!("{}:{}:{}", ips[last_index], listen_ports[last_index], domains[last_index]));

    execute_init_chain(InitChainOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone()
    });
    execute_init_chain_config(InitChainConfigOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        timestamp: opts.timestamp,
        prevhash: opts.prevhash,
        version: opts.version,
        chain_id: opts.chain_id,
        block_interval: opts.block_interval,
        block_limit: opts.block_limit,
        network_image: opts.network_image.clone(),
        network_tag: opts.network_tag,
        consensus_image: opts.consensus_image,
        consensus_tag: opts.consensus_tag,
        executor_image: opts.executor_image,
        executor_tag: opts.executor_tag,
        storage_image: opts.storage_image,
        storage_tag: opts.storage_tag,
        controller_image: opts.controller_image,
        controller_tag: opts.controller_tag,
        kms_image: opts.kms_image,
        kms_tag: opts.kms_tag
    });
    let is_tls = opts.network_image.clone().as_str() == NETWORK_TLS;

    if is_tls {
        execute_create_ca(CreateCAOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
        });
    }
    let mut nodes: Vec<(u64, String)>  = Vec::new();
    for _ in 0..(domains.len() + 1) {
        nodes.push(execute_new_account(NewAccountOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            kms_password: opts.kms_password.clone()
        }).unwrap());
    }
    execute_set_admin(SetAdminOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        admin: nodes[0].clone().1
    });

    let mut validators = String::from("");
    for i in 1..nodes.len() - 1 {
        validators = format!("{}{}", validators, format!("{}{}", nodes[i].1, ","));
    }
    validators = format!("{}{}", validators, nodes[nodes.len() - 1].1);
    execute_set_validators(SetValidatorsOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        validators: validators.to_string()
    });

    execute_set_nodelist(SetNodeListOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        node_list: node_list
    });


    for i in 0..domains.len() {
        let network_port = network_ports[i];
        let domain: String = domains[i].to_string();
        let listen_port: u16 = listen_ports[i];
        let node: (u64, String) = nodes[i + 1].clone();
        if is_tls {
            execute_create_csr(CreateCSROpts {
                chain_name: opts.chain_name.clone(),
                config_dir: opts.config_dir.clone(),
                domain: domain.clone()
            });
            execute_sign_csr(SignCSROpts {
                chain_name: opts.chain_name.clone(),
                config_dir: opts.config_dir.clone(),
                domain: domain.clone()
            });
        }
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
            network_listen_port: listen_port,
            kms_password: opts.kms_password.clone(),
            key_id: node.0,
            package_limit: opts.package_limit.clone(),
            log_level: opts.log_level.clone()
        });
        execute_update_node(UpdateNodeOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            config_name: opts.config_name.clone(),
            domain: domain.clone(),
            account: node.1
        });

    }


    Ok(())
}

mod create_new_test {
    use crate::create_new::{CreateLocalOpts, execute_create};

    #[test]
    fn create_test() {
        execute_create(CreateLocalOpts {
            chain_name: "test-chain".to_string(),
            config_dir: ".".to_string(),
            timestamp: 0,
            prevhash: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            version: 0,
            chain_id: "".to_string(),
            block_interval: 3,
            block_limit: 100,
            network_image: "network_p2p".to_string(),
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
            grpc_ports: "default".to_string(),
            node_list: "default".to_string(),
            p2p_listen_ports: "default".to_string(),
            peers_count: Some(2),
            config_name: "config.toml".to_string(),
            kms_password: "123456".to_string(),
            log_level: "info".to_string(),
            package_limit: 30000,
        });
    }
}
