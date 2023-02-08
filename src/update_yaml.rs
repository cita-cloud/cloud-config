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

use crate::constant::{
    CHAIN_CONFIG_FILE, CONSENSUS_OVERLORD, CONSENSUS_RAFT, CONTROLLER, CRYPTO_ETH, CRYPTO_SM,
    EXECUTOR_EVM, NETWORK_ZENOH, NODE_CONFIG_FILE, PRIVATE_KEY, STORAGE_ROCKSDB, VALIDATOR_ADDRESS,
};
use crate::error::Error;
use crate::util::{
    find_micro_service, read_chain_config, read_file, read_node_config, svc_name, write_file,
};
use clap::Parser;
use k8s_openapi::{
    api::{
        apps::v1::{StatefulSet, StatefulSetSpec},
        core::v1::{
            Affinity, ConfigMap, ConfigMapVolumeSource, Container, ContainerPort, ExecAction,
            HostAlias, PersistentVolumeClaim, PersistentVolumeClaimSpec, PodAffinityTerm,
            PodAntiAffinity, PodSecurityContext, PodSpec, PodTemplateSpec, Probe,
            ResourceRequirements, Service, ServicePort, ServiceSpec, Volume, VolumeMount,
            WeightedPodAffinityTerm,
        },
    },
    apimachinery::pkg::{
        api::resource::Quantity,
        apis::meta::v1::{LabelSelector, LabelSelectorRequirement, ObjectMeta},
        util::intstr::IntOrString,
    },
    ByteString,
};

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::net::Ipv4Addr;

/// A subcommand for run
#[derive(Parser, Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct UpdateYamlOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub config_dir: String,
    /// set config file name
    #[clap(long = "config-name", default_value = "config.toml")]
    pub config_name: String,
    /// domain of node
    #[clap(long = "domain")]
    pub domain: String,
    /// image pull policy: IfNotPresent or Always
    #[clap(long = "pull-policy", default_value = "IfNotPresent")]
    pub pull_policy: String,
    /// docker registry
    #[clap(long = "docker-registry", default_value = "docker.io")]
    pub docker_registry: String,
    /// docker repo
    #[clap(long = "docker-repo", default_value = "citacloud")]
    pub docker_repo: String,
    /// storage class
    #[clap(long = "storage-class")]
    pub storage_class: String,
    /// storage capacity
    #[clap(long = "storage-capacity", default_value = "10Gi")]
    pub storage_capacity: String,
    /// container resource requirements -- requests cpu
    #[clap(long = "requests-cpu", default_value = "10m")]
    pub requests_cpu: String,
    /// container resource requirements -- requests memory
    #[clap(long = "requests-memory", default_value = "32Mi")]
    pub requests_memory: String,
    /// container resource requirements -- limits cpu
    #[clap(long = "limits-cpu", default_value = "4000m")]
    pub limits_cpu: String,
    /// container resource requirements -- limits memory
    #[clap(long = "limits-memory", default_value = "8192Mi")]
    pub limits_memory: String,
    /// is enable debug
    #[clap(long = "enable-debug")]
    pub enable_debug: bool,
    /// is disable health-check
    #[clap(long = "disable-health-check")]
    pub disable_health_check: bool,
}

impl Default for UpdateYamlOpts {
    fn default() -> Self {
        Self {
            chain_name: "test-chain".to_string(),
            config_dir: ".".to_string(),
            config_name: "config.toml".to_string(),
            domain: Default::default(),
            pull_policy: "IfNotPresent".to_string(),
            docker_registry: "docker.io".to_string(),
            docker_repo: "citacloud".to_string(),
            storage_class: Default::default(),
            storage_capacity: "10Gi".to_string(),
            requests_cpu: "10m".to_string(),
            requests_memory: "32Mi".to_string(),
            limits_cpu: "4000m".to_string(),
            limits_memory: "8192Mi".to_string(),
            enable_debug: false,
            disable_health_check: false,
        }
    }
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct NodeK8sConfig {
    pub cm_config: ConfigMap,
    pub cm_log: ConfigMap,
    pub cm_account: ConfigMap,
    pub statefulset: StatefulSet,
    pub node_svc: Service,
}

/// generate k8s yaml by chain_config and node_config
pub fn execute_update_yaml(opts: UpdateYamlOpts) -> Result<NodeK8sConfig, Error> {
    let mut node_k8s_config = NodeK8sConfig::default();

    let node_name = format!("{}-{}", &opts.chain_name, &opts.domain);
    let node_dir = format!("{}/{}", &opts.config_dir, &node_name);

    // load node_config
    let file_name = format!("{}/{}", &node_dir, NODE_CONFIG_FILE);
    let node_config = read_node_config(file_name).unwrap();

    // load chain_config
    let file_name = format!("{}/{}", &node_dir, CHAIN_CONFIG_FILE);
    let chain_config = read_chain_config(file_name).unwrap();

    let config_file_name = format!("{}/{}", &node_dir, opts.config_name);

    let yamls_path = format!("{}/yamls", &node_dir);
    fs::create_dir_all(&yamls_path).unwrap();

    // protocol
    let network_protocol = if find_micro_service(&chain_config, NETWORK_ZENOH) {
        "UDP"
    } else {
        "TCP"
    };

    // update yaml
    // node svc
    // statefulset
    // configmap about config.toml
    // configmap about all log4rs.yaml
    // configmap about account
    {
        let mut metadata = ObjectMeta {
            name: Some(format!("{}-config", &node_name)),
            ..Default::default()
        };
        let mut labels = BTreeMap::new();
        labels.insert(
            "app.kubernetes.io/chain-name".to_string(),
            opts.chain_name.clone(),
        );
        labels.insert(
            "app.kubernetes.io/chain-node".to_string(),
            node_name.clone(),
        );
        metadata.labels = Some(labels);

        let mut cm_config = ConfigMap {
            metadata,
            ..Default::default()
        };
        let mut data = BTreeMap::new();
        data.insert(
            "config.toml".to_string(),
            read_file(config_file_name).unwrap(),
        );
        cm_config.data = Some(data);

        let yaml_file_name = format!("{}/cm-config.yaml", &yamls_path);
        write_file(
            serde_yaml::to_string(&cm_config).unwrap().as_bytes(),
            yaml_file_name,
        );
        node_k8s_config.cm_config = cm_config;
    }

    {
        let mut metadata = ObjectMeta {
            name: Some(format!("{}-log", &node_name)),
            ..Default::default()
        };
        let mut labels = BTreeMap::new();
        labels.insert(
            "app.kubernetes.io/chain-name".to_string(),
            opts.chain_name.clone(),
        );
        labels.insert(
            "app.kubernetes.io/chain-node".to_string(),
            node_name.clone(),
        );
        metadata.labels = Some(labels);

        let mut cm_log = ConfigMap {
            metadata,
            ..Default::default()
        };
        let mut data = BTreeMap::new();
        data.insert(
            "consensus-log4rs.yaml".to_string(),
            read_file(format!("{}/consensus-log4rs.yaml", &node_dir)).unwrap(),
        );
        data.insert(
            "controller-log4rs.yaml".to_string(),
            read_file(format!("{}/controller-log4rs.yaml", &node_dir)).unwrap(),
        );
        data.insert(
            "executor-log4rs.yaml".to_string(),
            read_file(format!("{}/executor-log4rs.yaml", &node_dir)).unwrap(),
        );
        data.insert(
            "crypto-log4rs.yaml".to_string(),
            read_file(format!("{}/crypto-log4rs.yaml", &node_dir)).unwrap(),
        );
        data.insert(
            "network-log4rs.yaml".to_string(),
            read_file(format!("{}/network-log4rs.yaml", &node_dir)).unwrap(),
        );
        data.insert(
            "storage-log4rs.yaml".to_string(),
            read_file(format!("{}/storage-log4rs.yaml", &node_dir)).unwrap(),
        );
        cm_log.data = Some(data);

        let yaml_file_name = format!("{}/cm-log.yaml", &yamls_path);
        write_file(
            serde_yaml::to_string(&cm_log).unwrap().as_bytes(),
            yaml_file_name,
        );
        node_k8s_config.cm_log = cm_log;
    }

    {
        let mut metadata = ObjectMeta {
            name: Some(format!("{}-account", &node_name)),
            ..Default::default()
        };
        let mut labels = BTreeMap::new();
        labels.insert(
            "app.kubernetes.io/chain-name".to_string(),
            opts.chain_name.clone(),
        );
        labels.insert(
            "app.kubernetes.io/chain-node".to_string(),
            node_name.clone(),
        );
        metadata.labels = Some(labels);

        let mut cm_account = ConfigMap {
            metadata,
            ..Default::default()
        };
        let mut data = BTreeMap::new();
        data.insert("node_address".to_string(), node_config.account);
        data.insert(
            "validator_address".to_string(),
            fs::read_to_string(format!("{}/{}", &node_dir, VALIDATOR_ADDRESS)).unwrap(),
        );
        cm_account.data = Some(data);

        let mut binary_data = BTreeMap::new();
        binary_data.insert(
            PRIVATE_KEY.to_string(),
            ByteString(fs::read(format!("{}/{}", &node_dir, PRIVATE_KEY)).unwrap()),
        );
        cm_account.binary_data = Some(binary_data);

        let yaml_file_name = format!("{}/cm-account.yaml", &yamls_path);
        write_file(
            serde_yaml::to_string(&cm_account).unwrap().as_bytes(),
            yaml_file_name,
        );
        node_k8s_config.cm_account = cm_account;
    }

    {
        let mut metadata = ObjectMeta {
            name: Some(node_name.clone()),
            ..Default::default()
        };
        let mut labels = BTreeMap::new();
        labels.insert(
            "app.kubernetes.io/chain-name".to_string(),
            opts.chain_name.clone(),
        );
        labels.insert(
            "app.kubernetes.io/chain-node".to_string(),
            node_name.clone(),
        );
        metadata.labels = Some(labels);

        let mut statefulset = StatefulSet {
            metadata,
            ..Default::default()
        };

        let mut spec = StatefulSetSpec::default();

        let mut selector = LabelSelector::default();
        let mut match_labels = BTreeMap::new();
        match_labels.insert(
            "app.kubernetes.io/chain-name".to_string(),
            opts.chain_name.clone(),
        );
        match_labels.insert(
            "app.kubernetes.io/chain-node".to_string(),
            node_name.clone(),
        );
        selector.match_labels = Some(match_labels);

        spec.selector = selector;

        spec.replicas = Some(1);

        let mut volume_claim_templates = Vec::new();

        let metadata = ObjectMeta {
            name: Some("datadir".to_string()),
            ..Default::default()
        };
        let mut pvc = PersistentVolumeClaim {
            metadata,
            ..Default::default()
        };
        let mut pvc_spec = PersistentVolumeClaimSpec {
            access_modes: Some(vec!["ReadWriteOnce".to_string()]),
            ..Default::default()
        };
        let mut resources = ResourceRequirements::default();
        let mut requests = BTreeMap::new();
        requests.insert(
            "storage".to_string(),
            Quantity(opts.storage_capacity.clone()),
        );
        resources.requests = Some(requests);
        pvc_spec.resources = Some(resources);
        pvc_spec.storage_class_name = Some(opts.storage_class.clone());
        pvc.spec = Some(pvc_spec);
        volume_claim_templates.push(pvc);
        spec.volume_claim_templates = Some(volume_claim_templates);

        let mut template = PodTemplateSpec::default();

        let mut metadata = ObjectMeta::default();
        let mut labels = BTreeMap::new();
        labels.insert(
            "app.kubernetes.io/chain-name".to_string(),
            opts.chain_name.clone(),
        );
        labels.insert(
            "app.kubernetes.io/chain-node".to_string(),
            node_name.clone(),
        );
        metadata.labels = Some(labels);
        template.metadata = Some(metadata);

        let mut node_cluster = String::default();
        for node_net_info in &chain_config.node_network_address_list {
            let real_domain = format!("{}-{}", &opts.chain_name, node_net_info.domain);
            if real_domain.eq(&node_name) {
                node_cluster = node_net_info.cluster.clone();
                break;
            }
        }

        let mut host_aliases: Vec<HostAlias> = vec![HostAlias {
            hostnames: Some(vec![node_name.clone()]),
            ip: Some("0.0.0.0".to_string()),
        }];

        for node_net_info in chain_config.node_network_address_list {
            if !node_cluster.eq(&node_net_info.cluster) {
                node_net_info
                    .host
                    .parse::<Ipv4Addr>()
                    .unwrap_or_else(|err| panic!("must be valid IP address: [{err}]"));
                host_aliases.push(HostAlias {
                    hostnames: Some(vec![format!(
                        "{}-{}",
                        &opts.chain_name, node_net_info.domain
                    )]),
                    ip: Some(node_net_info.host),
                })
            }
        }

        let mut template_spec = PodSpec {
            host_aliases: Some(host_aliases),
            security_context: Some(PodSecurityContext {
                run_as_user: Some(1000),
                run_as_group: Some(1000),
                fs_group: Some(1000),
                ..Default::default()
            }),
            affinity: Some(Affinity {
                pod_anti_affinity: Some(PodAntiAffinity {
                    preferred_during_scheduling_ignored_during_execution: Some(vec![
                        WeightedPodAffinityTerm {
                            weight: 100,
                            pod_affinity_term: PodAffinityTerm {
                                topology_key: "kubernetes.io/hostname".to_string(),
                                label_selector: Some(LabelSelector {
                                    match_expressions: Some(vec![LabelSelectorRequirement {
                                        key: "app.kubernetes.io/chain-name".to_string(),
                                        operator: "In".to_string(),
                                        values: Some(vec![opts.chain_name.clone()]),
                                    }]),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                        },
                    ]),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut containers = Vec::new();

        let mut container_resources_requirements = ResourceRequirements::default();
        let mut requests = BTreeMap::new();
        requests.insert("cpu".to_string(), Quantity(opts.requests_cpu.clone()));
        requests.insert("memory".to_string(), Quantity(opts.requests_memory.clone()));
        container_resources_requirements.requests = Some(requests);
        let mut limits = BTreeMap::new();
        limits.insert("cpu".to_string(), Quantity(opts.limits_cpu.clone()));
        limits.insert("memory".to_string(), Quantity(opts.limits_memory.clone()));
        container_resources_requirements.limits = Some(limits);

        // network
        let mut network_container = Container {
            name: "network".to_string(),
            image_pull_policy: Some(opts.pull_policy.clone()),
            ports: Some(vec![
                ContainerPort {
                    container_port: 40000,
                    name: Some("network".to_string()),
                    protocol: Some(network_protocol.to_string()),
                    ..Default::default()
                },
                ContainerPort {
                    container_port: 50000,
                    name: Some("grpc".to_string()),
                    protocol: Some("TCP".to_string()),
                    ..Default::default()
                },
            ]),
            volume_mounts: Some(vec![
                VolumeMount {
                    mount_path: "/data".to_string(),
                    name: "datadir".to_string(),
                    ..Default::default()
                },
                VolumeMount {
                    mount_path: "/etc/cita-cloud/config".to_string(),
                    name: "node-config".to_string(),
                    ..Default::default()
                },
                VolumeMount {
                    mount_path: "/etc/cita-cloud/log".to_string(),
                    name: "node-log".to_string(),
                    ..Default::default()
                },
                VolumeMount {
                    mount_path: "/mnt".to_string(),
                    name: "node-account".to_string(),
                    ..Default::default()
                },
            ]),
            working_dir: Some("/data".to_string()),
            liveness_probe: if opts.disable_health_check {
                None
            } else {
                Some(Probe {
                    exec: Some(ExecAction {
                        command: Some(vec![
                            "grpc_health_probe".to_string(),
                            "-addr=127.0.0.1:50000".to_string(),
                        ]),
                    }),
                    initial_delay_seconds: Some(30),
                    period_seconds: Some(10),
                    ..Default::default()
                })
            },
            resources: Some(container_resources_requirements.clone()),
            ..Default::default()
        };

        for micro_service in &chain_config.micro_service_list {
            if micro_service.image.starts_with("network") {
                if micro_service.image == NETWORK_ZENOH {
                    network_container.image = Some(format!(
                        "{}/{}/{}:{}",
                        &opts.docker_registry,
                        &opts.docker_repo,
                        &micro_service.image,
                        &micro_service.tag
                    ));
                    network_container.command = Some(vec![
                        "network".to_string(),
                        "run".to_string(),
                        "-c".to_string(),
                        "/etc/cita-cloud/config/config.toml".to_string(),
                        "-l".to_string(),
                        "/etc/cita-cloud/log/network-log4rs.yaml".to_string(),
                    ]);
                } else {
                    panic!("Unkonwn network service!");
                }
            }
        }

        containers.push(network_container);

        // consensus
        let mut consensus_container = Container {
            name: "consensus".to_string(),
            image_pull_policy: Some(opts.pull_policy.clone()),
            ports: Some(vec![ContainerPort {
                container_port: 50001,
                name: Some("grpc".to_string()),
                protocol: Some("TCP".to_string()),
                ..Default::default()
            }]),
            volume_mounts: Some(vec![
                VolumeMount {
                    mount_path: "/data".to_string(),
                    name: "datadir".to_string(),
                    ..Default::default()
                },
                VolumeMount {
                    mount_path: "/etc/cita-cloud/config".to_string(),
                    name: "node-config".to_string(),
                    ..Default::default()
                },
                VolumeMount {
                    mount_path: "/etc/cita-cloud/log".to_string(),
                    name: "node-log".to_string(),
                    ..Default::default()
                },
                VolumeMount {
                    mount_path: "/mnt".to_string(),
                    name: "node-account".to_string(),
                    ..Default::default()
                },
            ]),
            working_dir: Some("/data".to_string()),
            liveness_probe: if opts.disable_health_check {
                None
            } else {
                Some(Probe {
                    exec: Some(ExecAction {
                        command: Some(vec![
                            "grpc_health_probe".to_string(),
                            "-addr=127.0.0.1:50001".to_string(),
                        ]),
                    }),
                    initial_delay_seconds: Some(30),
                    period_seconds: Some(10),
                    ..Default::default()
                })
            },
            resources: Some(container_resources_requirements.clone()),
            ..Default::default()
        };

        for micro_service in &chain_config.micro_service_list {
            if micro_service.image.starts_with("consensus") {
                if micro_service.image == CONSENSUS_RAFT {
                    consensus_container.image = Some(format!(
                        "{}/{}/{}:{}",
                        &opts.docker_registry,
                        &opts.docker_repo,
                        &micro_service.image,
                        &micro_service.tag
                    ));
                    consensus_container.command = Some(vec![
                        "consensus".to_string(),
                        "run".to_string(),
                        "-c".to_string(),
                        "/etc/cita-cloud/config/config.toml".to_string(),
                        "--stdout".to_string(),
                    ]);
                } else if micro_service.image == CONSENSUS_OVERLORD {
                    consensus_container.image = Some(format!(
                        "{}/{}/{}:{}",
                        &opts.docker_registry,
                        &opts.docker_repo,
                        &micro_service.image,
                        &micro_service.tag
                    ));
                    consensus_container.command = Some(vec![
                        "consensus".to_string(),
                        "run".to_string(),
                        "-c".to_string(),
                        "/etc/cita-cloud/config/config.toml".to_string(),
                        "-l".to_string(),
                        "/etc/cita-cloud/log/consensus-log4rs.yaml".to_string(),
                        "-p".to_string(),
                        "/mnt/private_key".to_string(),
                    ]);
                } else {
                    panic!("Unkonwn consensus service!");
                }
            }
        }

        containers.push(consensus_container);

        // executor
        let mut executor_container = Container {
            name: "executor".to_string(),
            image_pull_policy: Some(opts.pull_policy.clone()),
            ports: Some(vec![ContainerPort {
                container_port: 50002,
                name: Some("grpc".to_string()),
                protocol: Some("TCP".to_string()),
                ..Default::default()
            }]),
            volume_mounts: Some(vec![
                VolumeMount {
                    mount_path: "/data".to_string(),
                    name: "datadir".to_string(),
                    ..Default::default()
                },
                VolumeMount {
                    mount_path: "/etc/cita-cloud/config".to_string(),
                    name: "node-config".to_string(),
                    ..Default::default()
                },
                VolumeMount {
                    mount_path: "/etc/cita-cloud/log".to_string(),
                    name: "node-log".to_string(),
                    ..Default::default()
                },
            ]),
            working_dir: Some("/data".to_string()),
            liveness_probe: if opts.disable_health_check {
                None
            } else {
                Some(Probe {
                    exec: Some(ExecAction {
                        command: Some(vec![
                            "grpc_health_probe".to_string(),
                            "-addr=127.0.0.1:50002".to_string(),
                        ]),
                    }),
                    initial_delay_seconds: Some(30),
                    period_seconds: Some(10),
                    ..Default::default()
                })
            },
            resources: Some(container_resources_requirements.clone()),
            ..Default::default()
        };

        for micro_service in &chain_config.micro_service_list {
            if micro_service.image.starts_with("executor") {
                if micro_service.image == EXECUTOR_EVM {
                    executor_container.image = Some(format!(
                        "{}/{}/{}:{}",
                        &opts.docker_registry,
                        &opts.docker_repo,
                        &micro_service.image,
                        &micro_service.tag
                    ));
                    executor_container.command = Some(vec![
                        "executor".to_string(),
                        "run".to_string(),
                        "-c".to_string(),
                        "/etc/cita-cloud/config/config.toml".to_string(),
                        "-l".to_string(),
                        "/etc/cita-cloud/log/executor-log4rs.yaml".to_string(),
                    ]);
                } else {
                    panic!("Unkonwn executor service!");
                }
            }
        }

        containers.push(executor_container);

        // storage
        let mut storage_container = Container {
            name: "storage".to_string(),
            image_pull_policy: Some(opts.pull_policy.clone()),
            ports: Some(vec![ContainerPort {
                container_port: 50003,
                name: Some("grpc".to_string()),
                protocol: Some("TCP".to_string()),
                ..Default::default()
            }]),
            volume_mounts: Some(vec![
                VolumeMount {
                    mount_path: "/data".to_string(),
                    name: "datadir".to_string(),
                    ..Default::default()
                },
                VolumeMount {
                    mount_path: "/etc/cita-cloud/config".to_string(),
                    name: "node-config".to_string(),
                    ..Default::default()
                },
                VolumeMount {
                    mount_path: "/etc/cita-cloud/log".to_string(),
                    name: "node-log".to_string(),
                    ..Default::default()
                },
            ]),
            working_dir: Some("/data".to_string()),
            liveness_probe: if opts.disable_health_check {
                None
            } else {
                Some(Probe {
                    exec: Some(ExecAction {
                        command: Some(vec![
                            "grpc_health_probe".to_string(),
                            "-addr=127.0.0.1:50003".to_string(),
                        ]),
                    }),
                    initial_delay_seconds: Some(30),
                    period_seconds: Some(10),
                    ..Default::default()
                })
            },
            resources: Some(container_resources_requirements.clone()),
            ..Default::default()
        };

        for micro_service in &chain_config.micro_service_list {
            if micro_service.image.starts_with("storage") {
                if micro_service.image == STORAGE_ROCKSDB {
                    storage_container.image = Some(format!(
                        "{}/{}/{}:{}",
                        &opts.docker_registry,
                        &opts.docker_repo,
                        &micro_service.image,
                        &micro_service.tag
                    ));
                    storage_container.command = Some(vec![
                        "storage".to_string(),
                        "run".to_string(),
                        "-c".to_string(),
                        "/etc/cita-cloud/config/config.toml".to_string(),
                        "-l".to_string(),
                        "/etc/cita-cloud/log/storage-log4rs.yaml".to_string(),
                    ]);
                } else {
                    panic!("Unkonwn storage service!");
                }
            }
        }

        containers.push(storage_container);

        // controller
        let mut controller_container = Container {
            name: "controller".to_string(),
            image_pull_policy: Some(opts.pull_policy.clone()),
            ports: Some(vec![ContainerPort {
                container_port: 50004,
                name: Some("grpc".to_string()),
                protocol: Some("TCP".to_string()),
                ..Default::default()
            }]),
            volume_mounts: Some(vec![
                VolumeMount {
                    mount_path: "/data".to_string(),
                    name: "datadir".to_string(),
                    ..Default::default()
                },
                VolumeMount {
                    mount_path: "/etc/cita-cloud/config".to_string(),
                    name: "node-config".to_string(),
                    ..Default::default()
                },
                VolumeMount {
                    mount_path: "/etc/cita-cloud/log".to_string(),
                    name: "node-log".to_string(),
                    ..Default::default()
                },
                VolumeMount {
                    mount_path: "/mnt".to_string(),
                    name: "node-account".to_string(),
                    ..Default::default()
                },
            ]),
            working_dir: Some("/data".to_string()),
            liveness_probe: if opts.disable_health_check {
                None
            } else {
                Some(Probe {
                    exec: Some(ExecAction {
                        command: Some(vec![
                            "grpc_health_probe".to_string(),
                            "-addr=127.0.0.1:50004".to_string(),
                        ]),
                    }),
                    initial_delay_seconds: Some(60),
                    period_seconds: Some(10),
                    ..Default::default()
                })
            },
            resources: Some(container_resources_requirements.clone()),
            ..Default::default()
        };

        for micro_service in &chain_config.micro_service_list {
            if micro_service.image.starts_with("controller") {
                if micro_service.image == CONTROLLER {
                    controller_container.image = Some(format!(
                        "{}/{}/{}:{}",
                        &opts.docker_registry,
                        &opts.docker_repo,
                        &micro_service.image,
                        &micro_service.tag
                    ));
                    controller_container.command = Some(vec![
                        "controller".to_string(),
                        "run".to_string(),
                        "-c".to_string(),
                        "/etc/cita-cloud/config/config.toml".to_string(),
                        "-l".to_string(),
                        "/etc/cita-cloud/log/controller-log4rs.yaml".to_string(),
                    ]);
                } else {
                    panic!("Unkonwn controller service!");
                }
            }
        }

        containers.push(controller_container);

        // crypto
        let mut crypto_container = Container {
            name: "crypto".to_string(),
            image_pull_policy: Some(opts.pull_policy.clone()),
            ports: Some(vec![ContainerPort {
                container_port: 50005,
                name: Some("grpc".to_string()),
                protocol: Some("TCP".to_string()),
                ..Default::default()
            }]),
            volume_mounts: Some(vec![
                VolumeMount {
                    mount_path: "/data".to_string(),
                    name: "datadir".to_string(),
                    ..Default::default()
                },
                VolumeMount {
                    mount_path: "/etc/cita-cloud/config".to_string(),
                    name: "node-config".to_string(),
                    ..Default::default()
                },
                VolumeMount {
                    mount_path: "/etc/cita-cloud/log".to_string(),
                    name: "node-log".to_string(),
                    ..Default::default()
                },
                VolumeMount {
                    mount_path: "/mnt".to_string(),
                    name: "node-account".to_string(),
                    ..Default::default()
                },
            ]),
            working_dir: Some("/data".to_string()),
            liveness_probe: if opts.disable_health_check {
                None
            } else {
                Some(Probe {
                    exec: Some(ExecAction {
                        command: Some(vec![
                            "grpc_health_probe".to_string(),
                            "-addr=127.0.0.1:50005".to_string(),
                        ]),
                    }),
                    initial_delay_seconds: Some(30),
                    period_seconds: Some(10),
                    ..Default::default()
                })
            },
            resources: Some(container_resources_requirements),
            ..Default::default()
        };

        for micro_service in &chain_config.micro_service_list {
            if micro_service.image == CRYPTO_ETH || micro_service.image == CRYPTO_SM {
                crypto_container.image = Some(format!(
                    "{}/{}/{}:{}",
                    &opts.docker_registry,
                    &opts.docker_repo,
                    &micro_service.image,
                    &micro_service.tag
                ));
                crypto_container.command = Some(vec![
                    "crypto".to_string(),
                    "run".to_string(),
                    "-c".to_string(),
                    "/etc/cita-cloud/config/config.toml".to_string(),
                    "-l".to_string(),
                    "/etc/cita-cloud/log/crypto-log4rs.yaml".to_string(),
                    "-p".to_string(),
                    "/mnt/private_key".to_string(),
                ]);
            }
        }

        containers.push(crypto_container);

        // debug
        if opts.enable_debug {
            let mut debug_container = Container {
                name: "debug".to_string(),
                image_pull_policy: Some(opts.pull_policy.clone()),
                ports: Some(vec![ContainerPort {
                    container_port: 9999,
                    name: Some("debug".to_string()),
                    protocol: Some("TCP".to_string()),
                    ..Default::default()
                }]),
                volume_mounts: Some(vec![
                    VolumeMount {
                        mount_path: "/data".to_string(),
                        name: "datadir".to_string(),
                        ..Default::default()
                    },
                    VolumeMount {
                        mount_path: "/etc/cita-cloud/config".to_string(),
                        name: "node-config".to_string(),
                        ..Default::default()
                    },
                    VolumeMount {
                        mount_path: "/etc/cita-cloud/log".to_string(),
                        name: "node-log".to_string(),
                        ..Default::default()
                    },
                    VolumeMount {
                        mount_path: "/mnt".to_string(),
                        name: "node-account".to_string(),
                        ..Default::default()
                    },
                ]),
                working_dir: Some("/data".to_string()),
                ..Default::default()
            };

            debug_container.image = Some("praqma/network-multitool:latest".to_string());

            containers.push(debug_container);
        }

        template_spec.containers = containers;

        template_spec.volumes = Some(vec![
            Volume {
                name: "node-account".to_string(),
                config_map: Some(ConfigMapVolumeSource {
                    name: Some(format!("{}-account", &node_name)),
                    ..Default::default()
                }),
                ..Default::default()
            },
            Volume {
                name: "node-config".to_string(),
                config_map: Some(ConfigMapVolumeSource {
                    name: Some(format!("{}-config", &node_name)),
                    ..Default::default()
                }),
                ..Default::default()
            },
            Volume {
                name: "node-log".to_string(),
                config_map: Some(ConfigMapVolumeSource {
                    name: Some(format!("{}-log", &node_name)),
                    ..Default::default()
                }),
                ..Default::default()
            },
        ]);

        template.spec = Some(template_spec);

        spec.template = template;
        statefulset.spec = Some(spec);

        let yaml_file_name = format!("{}/statefulset.yaml", &yamls_path);
        write_file(
            serde_yaml::to_string(&statefulset).unwrap().as_bytes(),
            yaml_file_name,
        );
        node_k8s_config.statefulset = statefulset;
    }

    {
        let mut node_svc = Service::default();

        let mut metadata = ObjectMeta {
            name: Some(svc_name(&opts.chain_name, &opts.domain)),
            ..Default::default()
        };
        let mut labels = BTreeMap::new();
        labels.insert(
            "app.kubernetes.io/chain-name".to_string(),
            opts.chain_name.clone(),
        );
        labels.insert(
            "app.kubernetes.io/chain-node".to_string(),
            node_name.clone(),
        );
        metadata.labels = Some(labels);

        node_svc.metadata = metadata;

        let mut svc_spec = ServiceSpec {
            type_: Some("ClusterIP".to_string()),
            ..Default::default()
        };

        let mut match_labels = BTreeMap::new();
        match_labels.insert("app.kubernetes.io/chain-name".to_string(), opts.chain_name);
        match_labels.insert("app.kubernetes.io/chain-node".to_string(), node_name);
        svc_spec.selector = Some(match_labels);

        svc_spec.ports = Some(vec![
            ServicePort {
                name: Some("network".to_string()),
                port: 40000,
                target_port: Some(IntOrString::Int(40000)),
                protocol: Some(network_protocol.to_string()),
                ..Default::default()
            },
            ServicePort {
                name: Some("rpc".to_string()),
                port: 50004,
                target_port: Some(IntOrString::Int(50004)),
                protocol: Some("TCP".to_string()),
                ..Default::default()
            },
            ServicePort {
                name: Some("call".to_string()),
                port: 50002,
                target_port: Some(IntOrString::Int(50002)),
                protocol: Some("TCP".to_string()),
                ..Default::default()
            },
        ]);

        node_svc.spec = Some(svc_spec);

        let yaml_file_name = format!("{}/node-svc.yaml", &yamls_path);
        write_file(
            serde_yaml::to_string(&node_svc).unwrap().as_bytes(),
            yaml_file_name,
        );
        node_k8s_config.node_svc = node_svc;
    }

    Ok(node_k8s_config)
}
