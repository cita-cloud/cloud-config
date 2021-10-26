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

use clap::Args;
use crate::error::{Error, Result};

/// A subcommand for run
#[derive(Args, Debug, Clone)]
pub struct AppendOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "tests-chain")]
    chain_name: String,
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

// fn init_admin(peers_count: usize, pair: &Vec<String>, opts: AppendOpts) -> AdminParam {
//     let path = if let Some(dir) = &opts.config_dir {
//         format!("{}/{}", dir, &opts.chain_name)
//     } else {
//         opts.chain_name.clone()
//     };
//     // let chain_path = path.as_str();
//     fs::create_dir_all(&path).unwrap();
//     let mut key_ids = Vec::new();
//     let mut addresses = Vec::new();
//     let mut uris = Vec::new();
//     let mut tls_peers: Vec<PeerConfig> = Vec::new();
//
//     let (ca_cert, ca_cert_pem, ca_key_pem) = ca_cert();
//
//     let mut f = File::create("ca_key.pem").unwrap();
//     f.write_all(ca_key_pem.as_bytes()).unwrap();
//     for i in 0..peers_count {
//         fs::create_dir_all(format!("{}-{}", path, i)).unwrap();
//
//         let (key_id, address) = key_pair(opts.get_dir(i as u16), opts.kms_password.clone());
//         key_ids.push(key_id);
//         addresses.push(format!("0x{}", hex::encode(address)));
//         let port: u16;
//         let ip: &str;
//         if !pair.is_empty() {
//             let mut v: Vec<&str> = pair[i].split(":").collect();
//             ip = v[0];
//             port = v[1].parse().unwrap();
//         } else {
//             ip = DEFAULT_ADDRESS;
//             port = P2P_PORT_BEGIN + i as u16;
//         }
//         uris.push(format!("/{}/{}/{}/{}", IPV4, ip, TCP, port));
//         let domain = format!("peer{}", i);
//         tls_peers.push(PeerConfig {
//             host: ip.into(),
//             port,
//             domain,
//         });
//     };
//     let mut file_name = format!("{}/{}", path.clone(), DEFAULT_CONFIG_NAME);
//     NetConfig::default(&uris).write(&file_name);
//     NetworkConfig::default(tls_peers.clone()).write(&file_name);
//     let genesis = GenesisBlock::default();
//     &genesis.write(&file_name);
//     let system = SystemConfigFile::default(
//         opts.version,
//         hex::encode(&opts.chain_name),
//         hex::encode("admin"),
//         addresses.clone());
//     &system.write(&file_name);
//     // admin account dir
//     let (admin_key, admin_address) = key_pair(opts.admin_dir(), opts.kms_password.clone());
//     let admin_address: String = format!("0x{}", hex::encode(admin_address));
//     AdminConfig::default(admin_key.clone(), admin_address.clone()).write(&file_name);
//     AdminParam {
//         admin_key,
//         admin_address,
//         chain_path: path.to_string(),
//         key_ids,
//         addresses,
//         uris,
//         tls_peers,
//         ca_cert,
//         ca_cert_pem,
//         genesis,
//         system,
//     }
// }
//
pub fn execute_append(opts: AppendOpts) -> Result {
//     match opts {
//         opts if opts.grpc_ports.is_empty() => match opts {
//             opts if opts.p2p_ports.is_empty() && opts.peers_count == None => return Err(Error::NodeCountNotExist),
//             opts if !opts.p2p_ports.is_empty() => match opts {
//                 opts if !validate_p2p_ports(opts.p2p_ports.clone()) => return Err(Error::P2pPortsParamNotValid),
//                 //以p2p_ports为准
//                 opts => {
//                     let pair: Vec<String> = opts.p2p_ports.split(",").map(String::from).collect();
//                     let peers_count = pair.len();
//                     let param = init_admin(peers_count, &pair, opts.clone());
//                     for i in 0..peers_count {
//                         let mut v: Vec<&str> = pair[i].split(":").collect();
//                         let rpc_port = GRPC_PORT_BEGIN + i as u16 * 1000;
//                         parse(opts.clone(), i, rpc_port, v[1].parse().unwrap(),
//                               param.admin_key,
//                               &param.admin_address,
//                               &param.chain_path,
//                               &param.key_ids,
//                               &param.addresses,
//                               &param.uris,
//                               &param.tls_peers,
//                               &param.ca_cert,
//                               &param.ca_cert_pem,
//                               &param.genesis,
//                               &param.system,
//                         )
//                     }
//                 }
//             },
//             //以peers_count为准
//             opts => {
//                 let peers_count: usize = opts.peers_count.unwrap() as usize;
//                 let param = init_admin(peers_count, &vec![], opts.clone());
//                 for i in 0..peers_count {
//                     let rpc_port = GRPC_PORT_BEGIN + i as u16 * 1000;
//                     let p2p_port = P2P_PORT_BEGIN + i as u16;
//                     parse(opts.clone(), i, rpc_port, p2p_port,
//                           param.admin_key,
//                           &param.admin_address,
//                           &param.chain_path,
//                           &param.key_ids,
//                           &param.addresses,
//                           &param.uris,
//                           &param.tls_peers,
//                           &param.ca_cert,
//                           &param.ca_cert_pem,
//                           &param.genesis,
//                           &param.system,
//                     )
//                 }
//             }
//         }
//         //以grpc_ports为准
//         opts if !opts.p2p_ports.is_empty() && opts.p2p_ports.split(",").count() != opts.grpc_ports.split(",").count() => return Err(Error::P2pPortsParamNotValid),
//         opts => {
//             let grpc_ports = opts.grpc_ports.split(",");
//             let peers_count = grpc_ports.count();
//             let param = init_admin(peers_count, &vec![], opts.clone());
//             for i in 0..peers_count {
//                 let rpc_ports: Vec<&str> = opts.grpc_ports.split(",").collect();
//                 let p2p_port = P2P_PORT_BEGIN + i as u16;
//
//                 parse(opts.clone(), i, rpc_ports[i].parse().unwrap(), p2p_port,
//                       param.admin_key,
//                       &param.admin_address,
//                       &param.chain_path,
//                       &param.key_ids,
//                       &param.addresses,
//                       &param.uris,
//                       &param.tls_peers,
//                       &param.ca_cert,
//                       &param.ca_cert_pem,
//                       &param.genesis,
//                       &param.system,
//                 )
//             }
//         }
//     };
    Ok(())
}
