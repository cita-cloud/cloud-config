use crate::append_node::{execute_append_node, AppendNodeOpts};
use crate::config::chain_config::NodeNetworkAddress;
use crate::constant::{DEFAULT_VALUE, NETWORK_TLS};
use crate::create_csr::{execute_create_csr, CreateCSROpts};
use crate::error::Error;
use crate::init_node::{execute_init_node, InitNodeOpts};
use crate::new_account::{execute_new_account, NewAccountOpts};
use crate::set_nodelist::{get_old_node_list_count, SetNodeListOpts};
use crate::sign_csr::{execute_sign_csr, SignCSROpts};
use crate::update_node::{execute_update_node, find_micro_service, UpdateNodeOpts};
use crate::util::read_chain_config;
use clap::Clap;
use std::iter::FromIterator;
use x509_parser::nom::lib::std::collections::HashSet;

/// A subcommand for run
#[derive(Clap, Debug, Clone)]
pub struct AppendNewOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    config_dir: String,
    /// set config file name
    #[clap(long = "config-name", default_value = "config.toml")]
    config_name: String,
    /// grpc port list, input "p1,p2,p3,p4", use default grpc port count from 50000 + 1000 * i
    /// use default must set peer_count or p2p_ports
    #[clap(long = "grpc-ports", default_value = "default")]
    grpc_ports: String,
    /// node list, input "localhost:40000:node0,localhost:40001:node1", use default port count from
    /// 127.0.0.1:40000 + 1 * i:nodei, use default must set peer_count or grpc_ports
    #[clap(long = "node-list", default_value = "default")]
    node_list: String,
    /// p2p listen port list, use default port count from 40000 + 1 * i ,use default must set
    /// peer_count or grpc_ports or p2p_ports
    #[clap(long = "p2p-listen-ports", default_value = "default")]
    p2p_listen_ports: String,
    /// set initial node number, default "none" mean not use this must set grpc_ports or p2p_ports,
    /// if set peers_count, grpc_ports and p2p_ports, base on grpc_ports > p2p_ports > peers_count
    #[clap(long = "peers-count")]
    peers_count: Option<u16>,
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

pub fn execute_append(opts: AppendNewOpts) -> Result<(), Error> {
    let mut domains: Vec<String> = Vec::new();
    let mut ips: Vec<String> = Vec::new();
    let mut listen_ports: Vec<u16> = Vec::new();
    let mut network_ports: Vec<u16> = Vec::new();
    let mut nodes_addresses: Vec<String> = Vec::new();
    let mut current_nodes = get_old_node_list_count(SetNodeListOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        node_list: "".to_string(),
    });
    current_nodes.reverse();
    let mut old_count: usize = 0;
    for node in current_nodes.clone() {
        if node.domain.starts_with("node") {
            old_count = node.domain.replace("node", "").parse::<usize>().unwrap() + 1;
            break;
        }
    }
    if opts.grpc_ports == DEFAULT_VALUE {
        if opts.node_list == DEFAULT_VALUE && opts.peers_count == None {
            return Err(Error::NodeCountNotExist);
        }
        if opts.node_list != DEFAULT_VALUE {
            let url: Vec<String> = opts.node_list.split(',').map(String::from).collect();
            for (i, item) in url.iter().enumerate() {
                nodes_addresses.push(item.clone());
                let pair: Vec<&str> = item.split(':').collect();
                let node = NodeNetworkAddress {
                    host: pair[0].into(),
                    port: pair[1].parse().unwrap(),
                    domain: pair[2].to_string(),
                };
                let node_set: HashSet<NodeNetworkAddress> =
                    HashSet::from_iter(current_nodes.clone());
                if node_set.contains(&node) {
                    return Err(Error::DupNodeName);
                }
                let index = i + old_count;
                domains.push(node.domain);
                listen_ports.push(node.port);
                network_ports.push((50000 + index * 1000) as u16);
                ips.push(node.host);
            }
        } else {
            let peers_count = opts.peers_count.unwrap() as usize;
            for i in 0..peers_count {
                let index = i + old_count;
                listen_ports.push((40000 + index) as u16);
                network_ports.push((50000 + index * 1000) as u16);
                domains.push(format!("node{}", index));
                ips.push("localhost".to_string());
                nodes_addresses.push(format!(
                    "{}:{}:node{}",
                    "localhost".to_string(),
                    (40000 + index) as u16,
                    index
                ));
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
                let index = i + old_count;
                listen_ports.push((40000 + index) as u16);
                domains.push(format!("node{}", index));
                ips.push("localhost".to_string());
                nodes_addresses.push(format!(
                    "{}:{}:node{}",
                    "localhost".to_string(),
                    (40000 + index) as u16,
                    index
                ));
            }
        } else {
            let node_list: Vec<String> = opts.node_list.split(',').map(String::from).collect();
            for (i, item) in node_list.iter().enumerate() {
                nodes_addresses.push(item.clone());
                let pair: Vec<&str> = item.split(':').collect();
                let node = NodeNetworkAddress {
                    host: pair[0].into(),
                    port: pair[1].parse().unwrap(),
                    domain: pair[2].to_string(),
                };
                let node_set: HashSet<NodeNetworkAddress> =
                    HashSet::from_iter(current_nodes.clone());
                if node_set.contains(&node) {
                    return Err(Error::DupNodeName);
                }
                let index = i + old_count;
                domains.push(node.domain);
                listen_ports.push(node.port);
                network_ports.push((50000 + index * 1000) as u16);
                ips.push(node.host);
            }
        }
    }

    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, "chain_config.toml"
    );
    let chain_config = read_chain_config(&file_name).unwrap();
    let is_tls = find_micro_service(&chain_config, NETWORK_TLS);

    let mut nodes: Vec<(u64, String)> = Vec::new();
    for _ in 0..(domains.len()) {
        nodes.push(
            execute_new_account(NewAccountOpts {
                chain_name: opts.chain_name.clone(),
                config_dir: opts.config_dir.clone(),
                kms_password: opts.kms_password.clone(),
            })
            .unwrap(),
        );
    }

    for i in 0..domains.len() {
        let network_port = network_ports[i];
        let listen_port: u16 = listen_ports[i];
        let domain: String = domains[i].to_string();
        let node: (u64, String) = nodes[i].clone();
        if is_tls {
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
            package_limit: opts.package_limit,
            log_level: opts.log_level.clone(),
        })
        .unwrap();
        execute_append_node(AppendNodeOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            node: nodes_addresses[i].clone(),
        })
        .unwrap();
        execute_update_node(UpdateNodeOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            config_name: opts.config_name.clone(),
            domain: domain.clone(),
            account: node.1,
        })
        .unwrap();
    }

    Ok(())
}

mod append_new_test {
    use crate::append_new::{execute_append, AppendNewOpts};

    #[test]
    fn append_test() {
        execute_append(AppendNewOpts {
            chain_name: "test-chain".to_string(),
            config_dir: ".".to_string(),
            config_name: "config.toml".to_string(),
            grpc_ports: "default".to_string(),
            node_list: "localhost:4000:hj1,localhost:4001:hn1".to_string(),
            p2p_listen_ports: "default".to_string(),
            peers_count: None,
            kms_password: "123456".to_string(),
            log_level: "info".to_string(),
            package_limit: 20000,
        })
        .unwrap();
    }
}
