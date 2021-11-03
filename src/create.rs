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

use crate::config::admin::{AdminConfig, AdminParam, CurrentConfig};
use crate::config::consensus_raft::Consensus;
use crate::config::controller::{ControllerConfig, GenesisBlock, SystemConfigFile};
use crate::config::executor_evm::ExecutorEvmConfig;
use crate::config::kms_sm::KmsSmConfig;
use crate::config::network_p2p::{NetConfig, PeerConfig};
use crate::config::network_tls::NetworkConfig;
use crate::config::storage_rocksdb::StorageRocksdbConfig;
use crate::constant::{
    CONSENSUS_BFT, CONSENSUS_RAFT, CONTROLLER, DEFAULT_ADDRESS, DEFAULT_CONFIG_NAME, DEFAULT_VALUE,
    EXECUTOR_EVM, GRPC_PORT_BEGIN, IPV4, KMS_SM, NETWORK_P2P, NETWORK_TLS, P2P_PORT_BEGIN,
    STORAGE_ROCKSDB, TCP,
};
use crate::error::Error;
use crate::traits::{Opts, TomlWriter, YmlWriter};
use crate::util::{ca_cert, cert, key_pair, validate_p2p_ports};
use clap::Clap;
use rcgen::{Certificate, CertificateParams, KeyPair};
use std::fs;
use rand::{Rng, thread_rng};

/// A subcommand for run
#[derive(Clap, Debug, Clone)]
pub struct CreateOpts {
    /// set version
    #[clap(long = "version", default_value = "0")]
    version: u32,
    /// set config file name
    #[clap(long = "config-name", default_value = "config.toml")]
    config_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir")]
    config_dir: Option<String>,
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    chain_name: String,
    /// Set controller micro-service.
    #[clap(long = "controller", default_value = "controller")]
    controller: String,
    /// Set consensus micro-service.
    #[clap(long = "consensus")]
    consensus: String,
    /// Set executor micro-service.
    #[clap(long = "executor", default_value = "executor_evm")]
    executor: String,
    /// Set network micro-service. network_tls or network_p2p.
    #[clap(long = "network")]
    network: String,
    /// Set kms micro-service.
    #[clap(long = "kms", default_value = "kms_sm")]
    kms: String,
    /// Set storage micro-service.
    #[clap(long = "storage", default_value = "storage_rocksdb")]
    storage: String,
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

impl CreateOpts {
    fn admin_dir(&self) -> String {
        if let Some(dir) = &self.config_dir {
            format!("{}/{}", dir, &self.chain_name)
        } else {
            (&self.chain_name).to_string()
        }
    }

    fn get_dir(&self, index: u16) -> String {
        if let Some(dir) = &self.config_dir {
            format!("{}/{}-{}", dir, &self.chain_name, index)
        } else {
            format!("{}-{}", &self.chain_name, index)
        }
    }
}

impl Opts for CreateOpts {
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
        // let chain_path = path.as_str();
        fs::create_dir_all(&path).unwrap();
        let mut key_ids = Vec::new();
        let mut addresses = Vec::new();
        let mut uris: Vec<PeerConfig> = Vec::new();
        let mut tls_peers: Vec<crate::config::network_tls::PeerConfig> = Vec::new();
        let mut grpc = Vec::new();
        let mut p2p = Vec::new();
        let mut ips = Vec::new();

        let is_tls = self.network == NETWORK_TLS;
        let is_p2p = self.network == NETWORK_P2P;
        let (ca_cert_pem, ca_key_pem) = if is_tls {
            let tuple = ca_cert();
            (tuple.1, tuple.2)
        } else if is_p2p {
            (String::from(""), String::from(""))
        } else {
            panic!("network only can choice network_p2p or network_tls")
        };


        for i in 0..peers_count {
            let dir = format!("{}-{}", path, i);
            fs::create_dir_all(&dir).unwrap();

            let (key_id, address) = key_pair(self.get_dir(i as u16), self.kms_password.clone());
            let address = hex::encode(address);
            let dir_new = format!("{}-{}", path, address);
            fs::rename(&dir, dir_new).unwrap();
            let address_str = format!("0x{}", address);
            key_ids.push(key_id);
            addresses.push(address_str.clone());
            if grpc_ports.is_empty() {
                grpc.push(GRPC_PORT_BEGIN + i as u16 * 1000)
            } else {
                grpc.push(grpc_ports[i])
            }
            let port: u16;
            let ip: &str;
            if !pair.is_empty() {
                let v: Vec<&str> = pair[i].split(':').collect();
                ip = v[0];
                port = v[1].parse().unwrap();
            } else {
                ip = DEFAULT_ADDRESS;
                port = P2P_PORT_BEGIN + i as u16;
            }
            ips.push(ip.to_string());
            p2p.push(port);
            if is_tls {
                tls_peers.push(crate::config::network_tls::PeerConfig {
                    host: ip.into(),
                    port,
                    domain: address_str,
                });
            } else if is_p2p {
                uris.push(PeerConfig {
                    address: format!("/{}/{}/{}/{}", IPV4, ip, TCP, port),
                });
            }
        }
        let file_name = format!("{}/{}", path, DEFAULT_CONFIG_NAME);
        if is_tls {
            NetworkConfig::default(tls_peers.clone()).write(&file_name);
        }
        if is_p2p {
            NetConfig::default(&uris).write(&file_name);
        }
        let genesis = GenesisBlock::default();
        genesis.write(&file_name);
        // admin account dir
        let (admin_key, admin_address) = key_pair(self.admin_dir(), self.kms_password.clone());
        let rand: [u8; 16] =  thread_rng().gen();
        let admin_address: String = format!("0x{}", hex::encode(admin_address));
        let system = SystemConfigFile::default(
            self.version,
            hex::encode(rand),
            admin_address.clone(),
            addresses.clone(),
        );
        system.write(&file_name);
        AdminConfig::new(admin_key, admin_address.clone()).write(&file_name);
        CurrentConfig::new(
            peers_count as u16,
            &uris,
            tls_peers.clone(),
            addresses.clone(),
            grpc.clone(),
            p2p.clone(),
            ips.clone(),
            ca_cert_pem.clone(),
            ca_key_pem.clone(),
        )
            .write(&file_name);
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
            count_old: 0,
        })
    }

    fn parse(&self, i: usize, admin: &AdminParam) {
        let address = &admin.addresses[i];
        let rm_0x = &admin.addresses[i][2..];
        let chain_name = format!("{}-{}", &admin.chain_path, rm_0x);
        let file_name = format!("{}/{}", &chain_name, self.config_name);
        let p2p_port = admin.p2p_ports[i];
        let rpc_port = admin.rpc_ports[i];
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
        let is_tls = self.network == NETWORK_TLS;
        let is_p2p = self.network == NETWORK_P2P;
        if is_p2p {
            if let Some(mut uris) = admin.uris.clone() {
                uris.remove(i);
                let config = NetConfig::new(p2p_port, rpc_port, &uris);
                config.write(&file_name);
                config.write_log4rs(&chain_name);
            }
        } else if is_tls {
            if let Some(mut tls_peers) = admin.tls_peers.clone() {
                tls_peers.remove(i);

                let ca_key_pair = KeyPair::from_pem(&admin.ca_key_pem).unwrap();
                let ca_param =
                    CertificateParams::from_ca_cert_pem(&admin.ca_cert_pem, ca_key_pair).unwrap();

                let (_, cert, priv_key) = cert(address, &Certificate::from_params(ca_param).unwrap());
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

pub fn execute_create(opts: CreateOpts) -> Result<(), Error> {
    if opts.controller.as_str() != CONTROLLER {
        return Err(Error::ControllerNotExist);
    }
    if opts.consensus.as_str() != CONSENSUS_BFT && opts.consensus.as_str() != CONSENSUS_RAFT {
        return Err(Error::ConsensusNotExist);
    }
    if opts.network.as_str() != NETWORK_P2P && opts.network.as_str() != NETWORK_TLS {
        return Err(Error::NetworkNotExist);
    }
    if opts.executor.as_str() != EXECUTOR_EVM {
        return Err(Error::ExecutorNotExist);
    }
    if opts.kms.as_str() != KMS_SM {
        return Err(Error::KmsNotDefaultOrKmsSm);
    }
    if opts.storage.as_str() != STORAGE_ROCKSDB {
        return Err(Error::StorageNotExist);
    }
    if opts.grpc_ports == DEFAULT_VALUE {
        if opts.p2p_ports == DEFAULT_VALUE && opts.peers_count == None {
            return Err(Error::NodeCountNotExist);
        }
        if opts.p2p_ports != DEFAULT_VALUE {
            if !validate_p2p_ports(opts.p2p_ports.clone()) {
                return Err(Error::P2pPortsParamNotValid);
            }
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
            if !validate_p2p_ports(opts.p2p_ports.clone()) {
                return Err(Error::P2pPortsParamNotValid);
            }
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
mod create_test {
    use super::*;
    use crate::constant::NETWORK_TLS;
    use crate::util::write_to_file;
    use toml::Value;

    #[test]
    fn create_test() {
        execute_create(CreateOpts {
            version: 0,
            config_name: DEFAULT_CONFIG_NAME.to_string(),
            config_dir: None,
            chain_name: "cita-chain".to_string(),
            controller: CONTROLLER.to_string(),
            consensus: CONSENSUS_BFT.to_string(),
            executor: EXECUTOR_EVM.to_string(),
            network: NETWORK_TLS.to_string(),
            kms: KMS_SM.to_string(),
            storage: STORAGE_ROCKSDB.to_string(),
            grpc_ports: "default".to_string(),
            p2p_ports: "default".to_string(),
            peers_count: Some(2),
            kms_password: "123456".to_string(),
            package_limit: 100,
        });
    }
}
