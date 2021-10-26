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
use std::path::Path;
use clap::Args;
use crate::config::admin::CurrentConfig;
use crate::constant::DEFAULT_CONFIG_NAME;
use crate::error::{Error, Result};
use crate::traits::TomlWriter;
use crate::util::{read_from_file, write_whole_to_file};

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


pub fn execute_delete(opts: DeleteOpts) -> Result {
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
    let mut file_name = format!("./{}/{}", path.clone(), DEFAULT_CONFIG_NAME);
    let mut config = read_from_file(&file_name).unwrap();
    fs::remove_file(&file_name);
    let mut index_set = HashSet::new();
    let current = config.current_config.as_ref().unwrap();
    if !opts.index.is_empty() {
        let a =  opts.index.split(",");
        for item in a {
            if item.parse::<usize>().is_err() {
                return Err(Error::DeleteParamNotValid);
            } else {
                index_set.insert(item.parse::<usize>().unwrap());
            }
        }
    } else {
        let a = opts.addresses.split(",");
        for item in a {
            match current.addresses.iter().position(|address| address == item) {
                None => return Err(Error::DeleteParamNotValid),
                Some(i) => match i {
                    i if i >= 2 => return Err(Error::DeleteParamNotValid),
                    i => {
                        index_set.insert(i);
                    }
                },
            }
        }
    }
    // let index = if let true = opts.index.is_empty() {
    //     index as usize
    // } else {
    //     let addresses = &config.current_config.as_ref().unwrap().addresses;
    //     addresses.iter().position(|x| {x == &opts.address.clone().unwrap()}).unwrap()
    // };

    for i in 0..current.addresses.len() {
        let chain_name = format!("./{}-{}", path, i);
        let file_name = format!("{}/{}", &chain_name, DEFAULT_CONFIG_NAME);
        if index_set.contains(&i) {
            fs::remove_dir_all(chain_name);
            continue;
        }
        let mut peer_config = read_from_file(&file_name).unwrap();
        fs::remove_file(&file_name);
        let mut peers = Vec::new();
        for (j, item) in current.peers.iter().enumerate() {
            if j != i && !index_set.contains(&j) {
                peers.push(item.clone());
            }
        }
        peer_config.network_p2p.peers = peers;
        let mut tls_peers = Vec::new();
        for (j, item) in current.tls_peers.iter().enumerate() {
            if j != i && !index_set.contains(&j) {
                tls_peers.push(item.clone());
            }
        }
        peer_config.network_tls.peers = tls_peers;
        println!("{:?}", peer_config);
        write_whole_to_file(peer_config, &file_name);
    }
    let mut current_new = current.clone();
    for index in index_set {
        current_new.peers.remove(index);
        current_new.tls_peers.remove(index);
        current_new.addresses.remove(index);
    }
    config.current_config = Some(current_new);
    write_whole_to_file(config, &file_name);
    // println!("{:?}", config);

    Ok(())
}

#[cfg(test)]
mod delete_test {
    use std::collections::HashMap;
    use std::convert::TryFrom;
    use super::*;
    use toml::Value;
    use crate::util::write_to_file;

    #[test]
    fn test_validator() {
        let a = vec![
            "0xdde07b7b74ed5dd715ee0f7cbe670bf09c8df274",
            "0x5df08441c932b361f36aabb4d7945294bea42732",
            "0xfd638ee4293e3b9fa37526d3ebbd64e1d4e11edf",
            "0x67fd1f568a0369d816a8f6ff0043da97c9d2f781",
        ];
        let index = a.iter().position(|x| {x == &"0x5df08841c932b361f36aabb4d7945294bea42732"});
        println!("{}", index.unwrap());
    }

    #[test]
    fn test_execute() {
        execute_delete(DeleteOpts {
            config_name: "".to_string(),
            config_dir: Some("".to_string()),
            chain_name: "cita-chain".to_string(),
            index: "0,1".to_string(),
            addresses: "".to_string(),
        });

    }
}


