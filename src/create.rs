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

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::RandomState;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::iter::FromIterator;
use clap::Args;
use rcgen::{BasicConstraints, Certificate, CertificateParams, IsCa, KeyPair, PKCS_ECDSA_P256_SHA256};
use regex::Regex;
use serde::Serialize;
use crate::traits::{Kms, TomlWriter};
use crate::error::{Error};
use crate::config::admin::{AdminConfig, CurrentConfig};
use crate::config::controller::{ControllerConfig, GenesisBlock, SystemConfigFile};
use crate::config::executor_evm::ExecutorConfig;
use crate::config::kms_sm::KmsConfig;
use crate::config::network_p2p::{NetConfig, PeerConfig};
use crate::config::network_tls::{NetworkConfig};
use crate::config::storage_rocksdb::StorageConfig;
use crate::constant::*;
type Result = std::result::Result<(), Error>;

/// A subcommand for run
#[derive(Args, Debug, Clone)]
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
    #[clap(long = "chain-name", default_value = "tests-chain")]
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
    /// Set network micro-service.
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
            format!("{}", &self.chain_name)
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

pub fn validate_p2p_ports(s: String) -> bool {
    match s {
        s if s.is_empty() => false,
        s => {
            let r = Regex::new(r"(^(\d|[1-9]\d|1\d{2}|2[0-4]\d|25[0-5])\.(\d|[1-9]\d|1\d{2}|2[0-4]\d|25[0-5])\.(\d|[1-9]\d|1\d{2}|2[0-4]\d|25[0-5])\.(\d|[1-9]\d|1\d{2}|2[0-4]\d|25[0-5]):([0-9]|[1-9]\d|[1-9]\d{2}|[1-9]\d{3}|[1-5]\d{4}|6[0-4]\d{3}|65[0-4]\d{2}|655[0-2]\d|6553[0-5])$)").unwrap();
            for item in s.split(",") {
                if !r.is_match(item) {
                    return false;
                }
            };
            true
        }
    }
}

pub fn key_pair(node_dir: String, kms_password: String) -> (u64, Vec<u8>) {
    let kms = crate::config::kms_sm::Kms::create_kms_db(format!("{}/{}", node_dir.clone(), "kms.db"), kms_password.clone());
    kms.generate_key_pair("create by cmd".to_string())
}

pub fn ca_cert() -> (Certificate, String, String) {
    let mut params = CertificateParams::new(vec![]);
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);

    let keypair = KeyPair::generate(&PKCS_ECDSA_P256_SHA256).unwrap();
    params.key_pair.replace(keypair);

    let cert = Certificate::from_params(params).unwrap();
    let cert_pem = cert.serialize_pem_with_signer(&cert).unwrap();
    let key_pem = cert.serialize_private_key_pem();
    (cert, cert_pem, key_pem)
}

pub fn cert(domain: &str, signer: &Certificate) -> (Certificate, String, String) {
    let subject_alt_names = vec![domain.into()];
    let mut params = CertificateParams::new(subject_alt_names);

    let keypair = KeyPair::generate(&PKCS_ECDSA_P256_SHA256).unwrap();
    params.key_pair.replace(keypair);

    let cert = Certificate::from_params(params).unwrap();
    let cert_pem = cert.serialize_pem_with_signer(signer).unwrap();
    let key_pem = cert.serialize_private_key_pem();
    (cert, cert_pem, key_pem)
}


fn parse(
    opts: CreateOpts,
    i: usize,
    admin: &AdminParam,
) {
    let chain_name = format!("{}-{}", &admin.chain_path, &admin.addresses[i][2..]);
    let file_name = format!("{}/{}", &chain_name, opts.config_name);
    let p2p_port = admin.p2p_ports[i];
    let rpc_port = admin.rpc_ports[i];
    let ip = admin.ips[i].clone();
    ControllerConfig::new(rpc_port, admin.key_ids[i], &admin.addresses[i], opts.package_limit).write(&file_name);
    admin.genesis.write(&file_name);
    admin.system.write(&file_name);

    let mut uris = admin.uris.clone();
    uris.remove(i);
    NetConfig::new(p2p_port, rpc_port, &uris).write(&file_name);

    let mut tls_peers = admin.tls_peers.clone();
    tls_peers.remove(i);
    let (_, cert, priv_key) = cert(&ip, &admin.ca_cert);
    NetworkConfig::new(p2p_port, rpc_port, admin.ca_cert_pem.clone(), cert, priv_key, tls_peers).write(&file_name);

    KmsConfig::new(rpc_port + 5).write(&file_name);
    AdminConfig::new(
        "0".to_string(),
        admin.admin_key,
        format!("{}/{}", chain_name, "kms.db"),
        format!("0x{}", hex::encode(admin.admin_address.clone()))).write(&file_name);
    StorageConfig::new(rpc_port + 5, rpc_port + 3).write(&file_name);
    ExecutorConfig::new(rpc_port + 2).write(&file_name);
}

pub struct AdminParam {
    pub admin_key: u64,
    pub admin_address: String,
    pub chain_path: String,
    pub key_ids: Vec<u64>,
    pub addresses: Vec<String>,
    pub uris: Vec<PeerConfig>,
    pub tls_peers: Vec<crate::config::network_tls::PeerConfig>,
    pub ca_cert: Certificate,
    pub ca_cert_pem: String,
    pub genesis: GenesisBlock,
    pub system: SystemConfigFile,
    pub rpc_ports: Vec<u16>,
    pub p2p_ports: Vec<u16>,
    pub ips: Vec<String>,
}

fn init_admin(peers_count: usize, pair: &Vec<String>, grpc_ports: Vec<u16>, opts: CreateOpts) -> AdminParam {
    let path = if let Some(dir) = &opts.config_dir {
        format!("{}/{}", dir, &opts.chain_name)
    } else {
        opts.chain_name.clone()
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

    let (ca_cert, ca_cert_pem, ca_key_pem) = ca_cert();

    let mut f = File::create("ca_key.pem").unwrap();
    f.write_all(ca_key_pem.as_bytes()).unwrap();
    for i in 0..peers_count {
        let dir = format!("{}-{}", path, i);
        fs::create_dir_all(&dir).unwrap();

        let (key_id, address) = key_pair(opts.get_dir(i as u16), opts.kms_password.clone());
        let address = hex::encode(address);
        let dir_new = format!("{}-{}", path, address);
        fs::rename(&dir, dir_new);

        key_ids.push(key_id);
        addresses.push(format!("0x{}", address));
        if grpc_ports.is_empty() {
            grpc.push(GRPC_PORT_BEGIN + i as u16 * 1000)
        } else {
            grpc.push(grpc_ports[i])
        }
        let port: u16;
        let ip: &str;
        if !pair.is_empty() {
            let mut v: Vec<&str> = pair[i].split(":").collect();
            ip = v[0];
            port = v[1].parse().unwrap();
        } else {
            ip = DEFAULT_ADDRESS;
            port = P2P_PORT_BEGIN + i as u16;
        }
        ips.push(ip.to_string());
        p2p.push(port.clone());

        uris.push(PeerConfig {
            address: format!("/{}/{}/{}/{}", IPV4, ip, TCP, port)
        });
        tls_peers.push(crate::config::network_tls::PeerConfig {
            host: ip.into(),
            port,
            domain: ip.into(),
        });
    };
    let mut file_name = format!("{}/{}", path.clone(), DEFAULT_CONFIG_NAME);
    NetConfig::default(&uris).write(&file_name);
    NetworkConfig::default(tls_peers.clone()).write(&file_name);
    let genesis = GenesisBlock::default();
    &genesis.write(&file_name);
    let system = SystemConfigFile::default(
        opts.version,
        hex::encode(&opts.chain_name),
        hex::encode("admin"),
        addresses.clone());
    &system.write(&file_name);
    // admin account dir
    let (admin_key, admin_address) = key_pair(opts.admin_dir(), opts.kms_password.clone());
    let admin_address: String = format!("0x{}", hex::encode(admin_address));
    AdminConfig::default(admin_key.clone(), admin_address.clone()).write(&file_name);
    CurrentConfig::new(&uris, tls_peers.clone(), addresses.clone(), grpc.clone(), p2p.clone(), ips.clone()).write(&file_name);
    AdminParam {
        admin_key,
        admin_address,
        chain_path: path.to_string(),
        key_ids,
        addresses,
        uris,
        tls_peers,
        ca_cert,
        ca_cert_pem,
        genesis,
        system,
        rpc_ports: grpc.clone(),
        p2p_ports: p2p.clone(),
        ips: ips.clone(),
    }
}


pub fn execute_create(opts: CreateOpts) -> Result {
    match opts {
        opts if opts.controller.as_str() != CONTROLLER => return Err(Error::ControllerNotExist),
        opts if opts.consensus.as_str() != CONSENSUS_BFT && opts.consensus.as_str() != CONSENSUS_RAFT => return Err(Error::ConsensusNotExist),
        opts if opts.executor.as_str() != EXECUTOR_EVM => return Err(Error::ExecutorNotExist),
        opts if opts.kms.as_str() != KMS_SM => return Err(Error::KmsNotDefaultOrKmsSm),
        opts if opts.storage.as_str() != STORAGE_ROCKSDB => return Err(Error::StorageNotExist),
        opts if opts.grpc_ports.is_empty() => match opts {
            opts if opts.p2p_ports.is_empty() && opts.peers_count == None => return Err(Error::NodeCountNotExist),
            opts if !opts.p2p_ports.is_empty() => match opts {
                opts if !validate_p2p_ports(opts.p2p_ports.clone()) => return Err(Error::P2pPortsParamNotValid),
                //以p2p_ports为准
                opts => {
                    let pair: Vec<String> = opts.p2p_ports.split(",").map(String::from).collect();
                    let peers_count = pair.len();
                    let param = init_admin(peers_count, &pair, vec![],opts.clone());
                    for i in 0..peers_count {
                        parse(opts.clone(), i, &param)
                    }
                }
            },
            //以peers_count为准
            opts => {
                let peers_count: usize = opts.peers_count.unwrap() as usize;
                let param = init_admin(peers_count, &vec![], vec![], opts.clone());
                for i in 0..peers_count {
                    parse(opts.clone(), i, &param)
                }
            }
        }
        //以grpc_ports为准
        opts if !opts.p2p_ports.is_empty() && opts.p2p_ports.split(",").count() != opts.grpc_ports.split(",").count() => return Err(Error::P2pPortsParamNotValid),
        opts => {
            let grpc_ports: Vec<u16> = opts.grpc_ports.split(",").map(|p| {p.parse().unwrap()}).collect();
            let pair: Vec<String> = opts.p2p_ports.split(",").map(String::from).collect();
            let peers_count = grpc_ports.len();
            let param = init_admin(peers_count, &pair, grpc_ports, opts.clone());
            for i in 0..peers_count {
                parse(opts.clone(), i,&param)
            }
        }
    };


    Ok(())
}

#[cfg(test)]
mod create_test {
    use super::*;
    use toml::Value;
    use crate::util::write_to_file;

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
            network: NETWORK_P2P.to_string(),
            kms: KMS_SM.to_string(),
            storage: STORAGE_ROCKSDB.to_string(),
            grpc_ports: "".to_string(),
            p2p_ports: "".to_string(),
            peers_count: Some(2),
            kms_password: "123456".to_string(),
            package_limit: 100,
        });
    }
}


