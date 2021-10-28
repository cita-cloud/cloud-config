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


use std::collections::HashSet;
use std::fs;
use std::iter::FromIterator;
use std::path::Path;
use crate::error::Error;
use crate::util::{read_from_file, write_whole_to_file};
use clap::Args;

/// A subcommand for run
#[derive(Args, Debug, Clone)]
pub struct DeleteOpts {
    /// set config file name
    #[clap(long = "config-name", default_value = "config.toml")]
    config_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir")]
    config_dir: Option<String>,
    /// set chain name
    #[clap(long = "chain-name", default_value = "tests-chain")]
    chain_name: String,
    /// delete index. Such as 1,2 will delete first and second node
    #[clap(long = "index")]
    index: String,
    /// delete node address. Such as 0x16e80b488f6e423b9faff014d1883493c5043d29,0x5bc21f512f877f18840abe13de5816c1226c4710 will node with the address
    #[clap(long = "addresses")]
    addresses: String,
}


pub fn execute_delete(opts: DeleteOpts) -> Result<(), Error> {
    let path = if let Some(dir) = &opts.config_dir {
        format!("{}/{}", dir, &opts.chain_name)
    } else {
        opts.chain_name.clone()
    };
    if !Path::new(format!("./{}", path).as_str()).exists() {
        return Err(Error::ConfigDirNotExist);
    }
    if opts.index.is_empty() && opts.addresses.is_empty() {
        return Err(Error::DeleteParamNotValid);
    }
    let file_name = format!("./{}/{}", path, opts.config_name);
    let mut config = read_from_file(&file_name).unwrap();
    let mut index_set = HashSet::new();
    let current = config.current_config.as_ref().unwrap();
    if !opts.index.is_empty() {
        let a =  opts.index.split(',');
        for item in a {
            if item.parse::<usize>().is_err() {
                return Err(Error::DeleteIndexNotValid);
            } else {
                index_set.insert(item.parse::<usize>().unwrap());
            }
        }
    } else {
        if opts.addresses.is_empty() {
            return Err(Error::DeleteParamNotValid);
        }
        let a = opts.addresses.split(',');
        for item in a {
            match current.addresses.iter().position(|address| address == item) {
                None => return Err(Error::DeleteParamNotValid),
                Some(i) => {
                    index_set.insert(i);
                }
            }
        }
    }
    let index_old = HashSet::from_iter(0..current.count as usize);
    if !index_old.intersection(&index_set).eq(&index_set) {
        return Err(Error::DeleteParamNotValid);
    }

    fs::remove_file(&file_name).unwrap();

    for i in 0..current.addresses.len() {

        let chain_name = format!("./{}-{}", path, &current.addresses[i][2..]);
        let file_name = format!("{}/{}", &chain_name, opts.config_name);
        if index_set.contains(&i) {
            fs::remove_dir_all(chain_name).unwrap();
            continue;
        }
        let mut peer_config = read_from_file(&file_name).unwrap();
        fs::remove_file(&file_name).unwrap();
        if let Some(p) = &current.peers {
            let mut peers = Vec::new();
            for (j, item) in p.iter().enumerate() {
                if j != i && !index_set.contains(&j) {
                    peers.push(item.clone());
                }
            }
            if let Some(mut net) = peer_config.network_p2p.as_mut() {
                net.peers = peers;
            }
        }

        if let Some(p) = &current.tls_peers {
            let mut tls_peers = Vec::new();
            for (j, item) in p.iter().enumerate() {
                if j != i && !index_set.contains(&j) {
                    tls_peers.push(item.clone());
                }
            }
            if let Some(mut net) = peer_config.network_tls.as_mut() {
                net.peers = tls_peers;
            }
        }

        println!("{:?}", peer_config);
        write_whole_to_file(peer_config, &file_name);
    }
    let mut current_new = current.clone();
    current_new.count -= index_set.len() as u16;
    if let Some(p) = &current_new.peers {
        let mut peers_new = Vec::new();
        for (i, peer) in p.iter().enumerate() {
            if !index_set.contains(&i) {
                peers_new.push(peer.clone());
            }
        }
        current_new.peers = Some(peers_new);
    }

    if let Some(p) = &current_new.tls_peers {
        let mut tls_peers_new = Vec::new();
        for (i, peer) in p.iter().enumerate() {
            if !index_set.contains(&i) {
                tls_peers_new.push(peer.clone());
            }
        }
        current_new.tls_peers = Some(tls_peers_new);
    }

    let mut addresses_new = Vec::new();
    for (i, address) in current_new.addresses.iter().enumerate() {
        if !index_set.contains(&i) {
            addresses_new.push(address.clone());
        }
    }
    current_new.addresses = addresses_new;
    let mut ips_new = Vec::new();
    for (i, ip) in current_new.ips.iter().enumerate() {
        if !index_set.contains(&i) {
            ips_new.push(ip.clone());
        }
    }
    current_new.ips = ips_new;

    let mut rpc_ports_new = Vec::new();
    for (i, rpc_port) in current_new.rpc_ports.iter().enumerate() {
        if !index_set.contains(&i) {
            rpc_ports_new.push(*rpc_port);
        }
    }
    current_new.rpc_ports = rpc_ports_new;

    let mut p2p_ports_new = Vec::new();
    for (i, p2p_port) in current_new.p2p_ports.iter().enumerate() {
        if !index_set.contains(&i) {
            p2p_ports_new.push(*p2p_port);
        }
    }
    current_new.p2p_ports = p2p_ports_new;

    config.current_config = Some(current_new);
    write_whole_to_file(config, &file_name);
    Ok(())
}

#[cfg(test)]
mod delete_test {

    use std::convert::TryFrom;
    use std::iter::FromIterator;
    use super::*;
    use toml::Value;
    use crate::util::write_to_file;

    #[test]
    fn test_set() {
        let a: HashSet<usize> = HashSet::from_iter([1, 3, 4]);
        let b: HashSet<usize> = HashSet::from_iter([3, 4]);
        println!("{:?}", a.intersection(&b).eq(&b));
    }

    #[test]
    fn test_validator() {
        let a = vec![
            "0xdde07b7b74ed5dd715ee0f7cbe670bf09c8df274",
            "0x5df08441c932b361f36aabb4d7945294bea42732",
            "0xfd638ee4293e3b9fa37526d3ebbd64e1d4e11edf",
            "0x67fd1f568a0369d816a8f6ff0043da97c9d2f781",
        ];
        let set: HashSet<usize> = HashSet::from_iter([2, 3]);
        let r: Vec<String> = a.into_iter().enumerate().flat_map(|(i, p)| {
            if !set.contains(&i) {
                Some(p.into())
            } else {
                None
            }
        }).collect();
        println!("...");
    }

    #[test]
    fn test_execute() {
        execute_delete(DeleteOpts {
            config_name: "config.toml".to_string(),
            config_dir: Some("".to_string()),
            chain_name: "cita-chain".to_string(),
            index: "1,3".to_string(),
            addresses: "".to_string(),
        });

    }
}


