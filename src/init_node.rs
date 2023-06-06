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

use crate::config::chain_config::ConfigStage;
use crate::config::node_config::{
    CloudStorageBuilder, GrpcPortsBuilder, MetricsPortsBuilder, NodeConfigBuilder,
};
use crate::constant::{ACCOUNT_DIR, CA_CERT_DIR, CERTS_DIR, CHAIN_CONFIG_FILE, NODE_CONFIG_FILE};
use crate::error::Error;
use crate::util::{copy_dir_all, read_chain_config, write_toml};
use clap::{ArgAction, Parser};
use std::fs;
use std::path::Path;

/// A subcommand for run
#[derive(Parser, Debug, Clone)]
pub struct InitNodeOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub(crate) chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub(crate) config_dir: String,
    /// domain of node
    #[clap(long = "domain")]
    pub(crate) domain: String,
    /// grpc network_port of node
    #[clap(long = "network-port", default_value = "50000")]
    pub(crate) network_port: u16,
    /// grpc consensus_port of node
    #[clap(long = "consensus-port", default_value = "50001")]
    pub(crate) consensus_port: u16,
    /// grpc executor_port of node
    #[clap(long = "executor-port", default_value = "50002")]
    pub(crate) executor_port: u16,
    /// grpc storage_port of node
    #[clap(long = "storage-port", default_value = "50003")]
    pub(crate) storage_port: u16,
    /// grpc controller_port of node
    #[clap(long = "controller-port", default_value = "50004")]
    pub(crate) controller_port: u16,
    /// network listen port of node
    #[clap(long = "network-listen-port", default_value = "40000")]
    pub(crate) network_listen_port: u16,
    /// log level
    #[clap(long = "log-level", default_value = "info")]
    pub(crate) log_level: String,
    /// log file path
    #[clap(long = "log-file-path")]
    pub(crate) log_file_path: Option<String>,
    /// jaeger agent endpoint
    #[clap(long = "jaeger-agent-endpoint")]
    pub(crate) jaeger_agent_endpoint: Option<String>,
    /// account of node
    #[clap(long = "account")]
    pub(crate) account: String,
    /// network metrics port of node
    #[clap(long = "network-metrics-port", default_value = "60000")]
    pub(crate) network_metrics_port: u16,
    /// consensus metrics port of node
    #[clap(long = "consensus-metrics-port", default_value = "60001")]
    pub(crate) consensus_metrics_port: u16,
    /// executor metrics port of node
    #[clap(long = "executor-metrics-port", default_value = "60002")]
    pub(crate) executor_metrics_port: u16,
    /// storage metrics port of node
    #[clap(long = "storage-metrics-port", default_value = "60003")]
    pub(crate) storage_metrics_port: u16,
    /// controller metrics port of node
    #[clap(long = "controller-metrics-port", default_value = "60004")]
    pub(crate) controller_metrics_port: u16,
    /// disable metrics
    #[clap(long = "disable-metrics", action = ArgAction::SetTrue)]
    pub(crate) disable_metrics: bool,
    /// is chain in danger mode
    #[clap(long = "is-danger", action = ArgAction::SetTrue)]
    pub(crate) is_danger: bool,
    /// cloud_storage.access_key_id
    #[clap(long = "access-key-id", default_value = "")]
    pub(crate) access_key_id: String,
    /// cloud_storage.secret_access_key
    #[clap(long = "secret-access-key", default_value = "")]
    pub(crate) secret_access_key: String,
    /// cloud_storage.endpoint
    #[clap(long = "s3-endpoint", default_value = "")]
    pub(crate) s3_endpoint: String,
    /// cloud_storage.bucket
    #[clap(long = "s3-bucket", default_value = "")]
    pub(crate) s3_bucket: String,
    /// cloud_storage.service_type: s3/oss(aliyun)/obs(huawei)/cos(tencent)/azblob(azure)
    #[clap(long = "service-type", default_value = "")]
    pub(crate) service_type: String,
    /// cloud_storage.root
    #[clap(long = "s3-root", default_value = "")]
    pub(crate) s3_root: String,
}

/// execute init node
pub fn execute_init_node(opts: InitNodeOpts) -> Result<(), Error> {
    let file_name = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CHAIN_CONFIG_FILE
    );

    if Path::new(&file_name).exists() {
        let chain_config = read_chain_config(&file_name).unwrap();
        // gen node config after chain config stage is Finalize
        if chain_config.stage != ConfigStage::Finalize {
            return Err(Error::InvalidStage);
        }
    } else {
        return Err(Error::InvalidStage);
    }

    let grpc_ports = GrpcPortsBuilder::default()
        .network_port(opts.network_port)
        .consensus_port(opts.consensus_port)
        .executor_port(opts.executor_port)
        .storage_port(opts.storage_port)
        .controller_port(opts.controller_port)
        .build();
    let metrics_ports = MetricsPortsBuilder::default()
        .network_metrics_port(opts.network_metrics_port)
        .consensus_metrics_port(opts.consensus_metrics_port)
        .executor_metrics_port(opts.executor_metrics_port)
        .storage_metrics_port(opts.storage_metrics_port)
        .controller_metrics_port(opts.controller_metrics_port)
        .build();
    let cloud_storage = CloudStorageBuilder::default()
        .access_key_id(opts.access_key_id.clone())
        .secret_access_key(opts.secret_access_key.clone())
        .endpoint(opts.s3_endpoint.clone())
        .bucket(opts.s3_bucket.clone())
        .service_type(opts.service_type.clone())
        .root(opts.s3_root.clone())
        .build();
    let node_config = NodeConfigBuilder::default()
        .grpc_ports(grpc_ports)
        .metrics_ports(metrics_ports)
        .network_listen_port(opts.network_listen_port)
        .log_level(opts.log_level)
        .log_file_path(opts.log_file_path)
        .jaeger_agent_endpoint(opts.jaeger_agent_endpoint)
        .account(opts.account)
        .enable_metrics(!opts.disable_metrics)
        .is_danger(opts.is_danger)
        .cloud_storage(cloud_storage)
        .build();

    let node_dir = format!("{}/{}-{}", &opts.config_dir, &opts.chain_name, &opts.domain);
    fs::create_dir_all(&node_dir).unwrap();

    // copy  accounts  ca_cert  certs and  chain_config.toml
    let from = format!("{}/{}/{}", &opts.config_dir, &opts.chain_name, ACCOUNT_DIR);
    let to = format!("{}/{}", &node_dir, ACCOUNT_DIR);
    copy_dir_all(from, to).unwrap();

    let from = format!("{}/{}/{}", &opts.config_dir, &opts.chain_name, CA_CERT_DIR);
    let to = format!("{}/{}", &node_dir, CA_CERT_DIR);
    copy_dir_all(from, to).unwrap();

    let from = format!("{}/{}/{}", &opts.config_dir, &opts.chain_name, CERTS_DIR);
    let to = format!("{}/{}", &node_dir, CERTS_DIR);
    copy_dir_all(from, to).unwrap();

    let from = format!(
        "{}/{}/{}",
        &opts.config_dir, &opts.chain_name, CHAIN_CONFIG_FILE
    );
    let to = format!("{}/{}", &node_dir, CHAIN_CONFIG_FILE);
    fs::copy(from, to).unwrap();

    let file_name = format!("{}/{}", &node_dir, NODE_CONFIG_FILE);
    write_toml(node_config, file_name);

    Ok(())
}
