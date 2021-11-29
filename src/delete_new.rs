use crate::constant::DEFAULT_VALUE;
use crate::delete_node::{execute_delete_folder, execute_set_node_list, DeleteNodeOpts};
use crate::error::Error;
use crate::set_nodelist::{get_old_node_list_count, SetNodeListOpts};
use crate::update_node::{execute_update_node, UpdateNodeOpts};
use crate::util::read_from_file;
use clap::Clap;
use std::collections::HashSet;
use std::iter::FromIterator;

/// A subcommand for run
#[derive(Clap, Debug, Clone)]
pub struct DeleteNewOpts {
    /// set config file name
    #[clap(long = "config-name", default_value = "config.toml")]
    config_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    config_dir: String,
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    chain_name: String,
    /// delete node address. Such as 0x16e80b488f6e423b9faff014d1883493c5043d29,0x5bc21f512f877f18840abe13de5816c1226c4710 will node with the address
    #[clap(long = "addresses", default_value = "default")]
    addresses: String,
}

pub fn execute_delete(opts: DeleteNewOpts) -> Result<(), Error> {
    let mut current_nodes = get_old_node_list_count(SetNodeListOpts {
        chain_name: opts.chain_name.clone(),
        config_dir: opts.config_dir.clone(),
        node_list: "".to_string(),
    });
    if opts.addresses == DEFAULT_VALUE {
        panic!("please input address to delete");
    }
    let input: Vec<String> = opts.addresses.split(',').map(String::from).collect();
    let domains: Vec<String> = current_nodes
        .iter()
        .map(|node| node.domain.clone())
        .collect();
    let input_set: HashSet<String> = HashSet::from_iter(input.clone());
    let domain_set: HashSet<String> = HashSet::from_iter(domains);
    if domain_set.intersection(&input_set).ne(&input_set) {
        panic!("input address doesn't exist!");
    }
    for domain in input_set {
        match current_nodes.binary_search_by(|node| node.domain.cmp(&domain)) {
            Ok(index) => {
                current_nodes.remove(index);
            }
            Err(_) => panic!("Can't found node that want to delete!"),
        }
    }

    for domain in input.clone() {
        execute_set_node_list(DeleteNodeOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            config_name: opts.config_name.clone(),
            domain,
        });
    }

    for node in current_nodes {
        let path = format!(
            "{}/{}-{}/{}",
            &opts.config_dir, &opts.chain_name, &node.domain, &opts.config_name
        );
        let config_toml = read_from_file(&path).unwrap();
        execute_update_node(UpdateNodeOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            config_name: opts.config_name.clone(),
            domain: node.domain,
            account: config_toml.controller.unwrap().node_address,
        })
        .unwrap();
    }

    for domain in input.clone() {
        execute_delete_folder(DeleteNodeOpts {
            chain_name: opts.chain_name.clone(),
            config_dir: opts.config_dir.clone(),
            config_name: opts.config_name.clone(),
            domain,
        });
    }

    Ok(())
}

mod delete_new_test {
    use crate::delete_new::{execute_delete, DeleteNewOpts};

    #[test]
    fn delete_test() {
        execute_delete(DeleteNewOpts {
            chain_name: "test-chain".to_string(),
            config_dir: ".".to_string(),
            config_name: "config.toml".to_string(),
            addresses: "hj,hj1".to_string(),
        })
        .unwrap();
    }
}
