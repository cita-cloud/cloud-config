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

use status_code::StatusCode;

/// A subcommand for run
#[derive(Clap, Debug)]
pub struct CreateOpts {
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
    #[clap(long = "peers-count", default_value = "none")]
    peers_count: u64,
    /// kms db password, default mean "123456"
    #[clap(long = "kms-password", default_value = "default")]
    kms_password: String,
    /// set one block contains tx limit, default 30000
    #[clap(long = "package-limit", default_value = "30000")]
    package_limit: u64,
}

pub fn execute_create(opts: CreateOpts) -> StatusCode {

    StatusCode::Success
}
