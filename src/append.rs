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

use crate::config::admin::AdminParam;
use crate::config::consensus_raft::Consensus;
use crate::config::controller::ControllerConfig;
use crate::config::executor_evm::ExecutorEvmConfig;
use crate::config::kms_sm::KmsSmConfig;
use crate::config::network_p2p::{NetConfig, PeerConfig};
use crate::config::network_tls::NetworkConfig;
use crate::config::storage_rocksdb::StorageRocksdbConfig;
use crate::constant::{DEFAULT_ADDRESS, DEFAULT_VALUE, DNS4, TCP};
use crate::error::Error;
use crate::traits::{Opts, TomlWriter, YmlWriter};
use crate::util::{cert, key_pair, read_from_file, write_whole_to_file};
use clap::Clap;
use rcgen::{Certificate, CertificateParams, KeyPair};
use std::fs;

/// A subcommand for run
#[derive(Clap, Debug, Clone)]
pub struct AppendOpts {
    /// set config file directory, default means current directory
    #[clap(long = "config-dir")]
    config_dir: Option<String>,
    /// set config file name
    #[clap(long = "config-name", default_value = "config.toml")]
    config_name: String,
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    chain_name: String,
    /// Set network micro-service.
    #[clap(long = "network", default_value = "network_p2p")]
    network: String,
    /// grpc port list, input "p1,p2,p3,p4", use default grpc port count from 50000 + 1000 * i
    /// use default must set peer_count or p2p_ports
    #[clap(long = "grpc-ports", default_value = "default")]
    grpc_ports: String,
    /// p2p port list, input "ip1:port1,ip2:port2,ip3:port3,ip4:port4", use default port count from
    /// 127.0.0.1:40000 + 1 * i, use default must set peer_count or grpc_ports
    #[clap(long = "p2p-ports", default_value = "default")]
    p2p_ports: String,
    /// set initial node number, default "none" mean not use this must set grpc_ports or p2p_ports,
    /// if set peers_count, grpc_ports and p2p_ports, base on grpc_ports > p2p_ports > peers_count
    #[clap(long = "peers-count")]
    peers_count: Option<u16>,
    /// kms db password
    #[clap(long = "kms-password", default_value = "123456")]
    kms_password: String,
    /// set one block contains tx limit, default 30000
    #[clap(long = "package-limit", default_value = "30000")]
    package_limit: u64,
}

impl AppendOpts {
    fn get_dir(&self, index: u16) -> String {
        if let Some(dir) = &self.config_dir {
            format!("{}/{}-{}", dir, &self.chain_name, index)
        } else {
            format!("{}-{}", &self.chain_name, index)
        }
    }
}

impl Opts for AppendOpts {
    fn init_admin(
        &self,
        peers_count: usize,
        pair: &[String],
        grpc_ports: Vec<u16>,
    ) -> Result<AdminParam, Error> {
        let path = if let Some(dir) = &self.config_dir {
            format!("{}/{}", dir, &self.chain_name)
        } else {
            self.chain_name.clone()
        };
        let file_name = format!("{}/{}", path, self.config_name);
        let mut config = read_from_file(&file_name).unwrap();
        fs::remove_file(&file_name).unwrap();

        let current = config.current_config.unwrap();

        let mut key_ids = Vec::new();
        let mut addresses = Vec::new();
        let mut addresses_inner = current.addresses.clone();
        let mut uris = current.peers.clone().unwrap_or_default();
        let mut tls_peers = current.tls_peers.clone().unwrap_or_default();
        let grpc_old = current.rpc_ports[current.rpc_ports.len() - 1];
        let p2p_old = current.p2p_ports[current.p2p_ports.len() - 1];
        let mut grpc = current.rpc_ports.clone();
        let mut p2p = current.p2p_ports.clone();
        let mut ips = current.ips.clone();
        let ca_cert_pem = current.ca_cert_pem.clone();
        let ca_key_pem = current.ca_key_pem.clone();

        for i in 0..peers_count {
            let rpc_port;
            if grpc_ports.is_empty() {
                rpc_port = grpc_old + (i + 1) as u16 * 1000;
            } else {
                rpc_port = grpc_ports[i];
            }
            // for item in &current.rpc_ports {
            //     if item == &rpc_port {
            //         return Err(Error::GrpcPortsParamNotValid);
            //     }
            // }
            grpc.push(rpc_port);
            let port: u16;
            let ip: &str;
            if !pair.is_empty() {
                let v: Vec<&str> = pair[i].split(':').collect();
                ip = v[0];
                port = v[1].parse().unwrap();
            } else {
                ip = DEFAULT_ADDRESS;
                port = p2p_old + (i + 1) as u16;
            }
            // for item in &current.p2p_ports {
            //     if item == &port {
            //         return Err(Error::P2pPortsParamNotValid);
            //     }
            // }
            ips.push(ip.to_string());
            p2p.push(port);

            let dir = format!("{}-{}", path, i + current.count as usize);
            fs::create_dir_all(&dir).unwrap();

            let (key_id, address) = key_pair(
                self.get_dir(i as u16 + current.count),
                self.kms_password.clone(),
            );
            let address = hex::encode(address);
            if !current.use_num {
                let dir_new = format!("{}-{}", path, &address);
                fs::rename(&dir, dir_new).unwrap();
            }

            let address_str = format!("0x{}", address);
            key_ids.push(key_id);
            addresses.push(address_str.clone());
            addresses_inner.push(address_str.clone());

            if !uris.is_empty() {
                uris.push(PeerConfig {
                    address: format!("/{}/{}/{}/{}", DNS4, ip, TCP, port),
                });
            }

            if !tls_peers.is_empty() {
                tls_peers.push(crate::config::network_tls::PeerConfig {
                    host: ip.into(),
                    port,
                    domain: address_str,
                });
            }
        }
        //the old
        for i in 0..current.addresses.len() {
            let (_chain_name, file_name) = if current.use_num {
                let node_dir = self.get_dir(i as u16);
                (
                    node_dir.clone(),
                    format!("{}/{}", &node_dir, self.config_name),
                )
            } else {
                let node_dir = format!("{}-{}", &path, &current.addresses[i][2..]);
                (
                    node_dir.clone(),
                    format!("{}/{}", &node_dir, self.config_name),
                )
            };

            let mut peer_config = read_from_file(&file_name).unwrap();
            fs::remove_file(&file_name).unwrap();
            let mut net = uris.clone();
            if !net.is_empty() {
                net.remove(i);
                if let Some(mut p2p) = peer_config.network_p2p.as_mut() {
                    p2p.peers = net;
                }
            }

            let mut tls_net = tls_peers.clone();
            if !tls_net.is_empty() {
                tls_net.remove(i);
                if let Some(mut tls) = peer_config.network_tls.as_mut() {
                    tls.peers = tls_net;
                }
            }
            write_whole_to_file(peer_config, &file_name);
        }
        let mut current_new = current.clone();
        let count_old = current_new.count;

        current_new.count += peers_count as u16;
        current_new.peers = Some(uris.clone());
        current_new.tls_peers = Some(tls_peers.clone());
        current_new.addresses = addresses_inner;
        current_new.ips = ips.clone();
        current_new.rpc_ports = grpc.clone();
        current_new.p2p_ports = p2p.clone();
        config.current_config = Some(current_new);
        write_whole_to_file(config.clone(), &file_name);

        let genesis = config.genesis_block.clone();
        let system = config.system_config.clone();
        let admin = config.admin_config.unwrap();
        // admin account dir
        let (admin_key, admin_address) = (admin.key_id, admin.admin_address);
        Ok(AdminParam {
            admin_key,
            admin_address,
            chain_path: path,
            key_ids,
            addresses,
            uris: Some(uris),
            tls_peers: Some(tls_peers),
            ca_cert_pem,
            ca_key_pem,
            genesis,
            system,
            rpc_ports: grpc,
            p2p_ports: p2p,
            ips,
            count_old,
            use_num: current.use_num,
        })
    }

    fn parse(&self, i: usize, admin: &AdminParam) {
        let address = &admin.addresses[i];
        let (chain_name, file_name) = if admin.use_num {
            let node_dir = format!("{}-{}", &admin.chain_path, i + admin.count_old as usize);
            (
                node_dir.clone(),
                format!("{}/{}", &node_dir, self.config_name),
            )
        } else {
            let rm_0x = &admin.addresses[i][2..];
            let node_dir = format!("{}-{}", &admin.chain_path, rm_0x);
            (
                node_dir.clone(),
                format!("{}/{}", &node_dir, self.config_name),
            )
        };

        let index = i + admin.count_old as usize;
        let p2p_port = admin.p2p_ports[index];
        let rpc_port = admin.rpc_ports[index];

        let controller = ControllerConfig::new(
            rpc_port,
            admin.key_ids[i],
            &admin.addresses[i],
            self.package_limit,
        );
        controller.write(&file_name);
        controller.write_log4rs(&chain_name);
        let consensus = Consensus::new(rpc_port, admin.addresses[i].clone());
        consensus.write(&file_name);
        consensus.write_log4rs(&chain_name);

        admin.genesis.write(&file_name);
        admin.system.write(&file_name);

        if !admin.uris.as_ref().unwrap().is_empty() {
            if let Some(mut uris) = admin.uris.clone() {
                uris.remove(index);
                let config = NetConfig::new(p2p_port, rpc_port, &uris);
                config.write(&file_name);
                config.write_log4rs(&chain_name);
            }
        } else if !admin.tls_peers.as_ref().unwrap().is_empty() {
            if let Some(mut tls_peers) = admin.tls_peers.clone() {
                tls_peers.remove(index);
                let ca_key_pair = KeyPair::from_pem(&admin.ca_key_pem).unwrap();
                let ca_param =
                    CertificateParams::from_ca_cert_pem(&admin.ca_cert_pem, ca_key_pair).unwrap();

                let (_, cert, priv_key) =
                    cert(address, &Certificate::from_params(ca_param).unwrap());
                let config = NetworkConfig::new(
                    p2p_port,
                    rpc_port,
                    admin.ca_cert_pem.clone(),
                    cert,
                    priv_key,
                    tls_peers,
                );
                config.write(&file_name);
                config.write_log4rs(&chain_name);
            }
        }
        let kms = KmsSmConfig::new(rpc_port + 5);
        kms.write(&file_name);
        kms.write_log4rs(&chain_name);

        let storage = StorageRocksdbConfig::new(rpc_port + 5, rpc_port + 3);
        storage.write(&file_name);
        storage.write_log4rs(&chain_name);
        let executor = ExecutorEvmConfig::new(rpc_port + 2);
        executor.write(&file_name);
        executor.write_log4rs(&chain_name);
    }
}

pub fn execute_append(opts: AppendOpts) -> Result<(), Error> {
    if opts.grpc_ports == DEFAULT_VALUE {
        if opts.p2p_ports == DEFAULT_VALUE && opts.peers_count == None {
            return Err(Error::NodeCountNotExist);
        }
        if opts.p2p_ports != DEFAULT_VALUE {
            // if !validate_p2p_ports(opts.p2p_ports.clone()) {
            //     return Err(Error::P2pPortsParamNotValid);
            // }
            let pair: Vec<String> = opts.p2p_ports.split(',').map(String::from).collect();
            let peers_count = pair.len();
            let param = opts.init_admin(peers_count, &pair, vec![]);
            match param {
                Ok(p) => {
                    for i in 0..peers_count {
                        opts.parse(i, &p)
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        } else {
            let peers_count: usize = opts.peers_count.unwrap() as usize;
            let param = opts.init_admin(peers_count, &[], vec![]);
            match param {
                Ok(p) => {
                    for i in 0..peers_count {
                        opts.parse(i, &p)
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    } else {
        if opts.p2p_ports != DEFAULT_VALUE
            && opts.p2p_ports.split(',').count() != opts.grpc_ports.split(',').count()
        {
            return Err(Error::P2pPortsParamNotValid);
        }
        let temp_ports: Vec<String> = opts.grpc_ports.split(',').map(String::from).collect();
        let mut grpc_ports: Vec<u16> = Vec::new();
        for item in temp_ports {
            if item.parse::<u16>().is_err() {
                return Err(Error::GrpcPortsParamNotValid);
            }
            grpc_ports.push(item.parse().unwrap());
        }
        let pair;
        if opts.p2p_ports == DEFAULT_VALUE {
            pair = vec![];
        } else {
            // if !validate_p2p_ports(opts.p2p_ports.clone()) {
            //     return Err(Error::P2pPortsParamNotValid);
            // }
            pair = opts.p2p_ports.split(',').map(String::from).collect();
        }
        let peers_count = grpc_ports.len();
        let param = opts.init_admin(peers_count, &pair, grpc_ports);
        match param {
            Ok(p) => {
                for i in 0..peers_count {
                    opts.parse(i, &p)
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod append_test {
    use super::*;
    use crate::util::write_to_file;
    use std::convert::TryFrom;
    use toml::Value;

    #[test]
    fn test_execute() {
        execute_append(AppendOpts {
            config_dir: None,
            config_name: String::from("config.toml"),
            chain_name: String::from("cita-chain"),
            network: String::from("network_p2p"),
            grpc_ports: String::from("default"),
            p2p_ports: String::from("default"),
            peers_count: Some(2),
            kms_password: String::from("123456"),
            package_limit: 30000,
        });
    }
}
