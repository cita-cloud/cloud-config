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
    CHAIN_CONFIG_FILE, CONSENSUS_OVERLORD, CONSENSUS_RAFT, CONTROLLER, CONTROLLER_HSM,
    EXECUTOR_EVM, NETWORK_ZENOH, NODE_CONFIG_FILE, PRIVATE_KEY, STORAGE_OPENDAL, VALIDATOR_ADDRESS,
};
use crate::error::Error;
use crate::util::{read_chain_config, read_file, read_node_config, svc_name, write_file};
use clap::Parser;
use k8s_openapi::{
    api::{
        apps::v1::{StatefulSet, StatefulSetSpec},
        core::v1::{
            Affinity, ConfigMap, ConfigMapVolumeSource, Container, ContainerPort, ExecAction,
            HostPathVolumeSource, PersistentVolumeClaim, PersistentVolumeClaimSpec,
            PodAffinityTerm, PodAntiAffinity, PodSecurityContext, PodSpec, PodTemplateSpec, Probe,
            ResourceRequirements, Service, ServicePort, ServiceSpec, Volume, VolumeMount,
            WeightedPodAffinityTerm,
        },
        discovery::v1::{Endpoint, EndpointPort, EndpointSlice},
    },
    apimachinery::pkg::{
        api::resource::Quantity,
        apis::meta::v1::{LabelSelector, LabelSelectorRequirement, ObjectMeta},
        util::intstr::IntOrString,
    },
    ByteString,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    net::{Ipv4Addr, Ipv6Addr},
};

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
    /// pvc access mode: ReadWriteOnce/ReadWriteMany
    #[clap(long = "access-mode", default_value = "ReadWriteMany")]
    pub access_mode: String,
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
    /// is gen kustomization
    #[clap(long = "enable-kustomize")]
    pub enable_kustomize: bool,
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
            access_mode: "ReadWriteMany".to_string(),
            storage_capacity: "10Gi".to_string(),
            requests_cpu: "10m".to_string(),
            requests_memory: "32Mi".to_string(),
            limits_cpu: "4000m".to_string(),
            limits_memory: "8192Mi".to_string(),
            enable_debug: false,
            disable_health_check: false,
            enable_kustomize: false,
        }
    }
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct NodeK8sConfig {
    pub cm_config: ConfigMap,
    pub cm_account: ConfigMap,
    pub statefulset: StatefulSet,
    pub node_svc: Service,
    pub external_svc: Vec<Service>,
    pub external_endpoints: Vec<EndpointSlice>,
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

    // check current node is k8s or not
    let mut my_cluster_name = "";
    let mut my_external_port = 0;
    let mut my_name_space = "";
    for node_network_address in &chain_config.node_network_address_list {
        if node_network_address.domain == opts.domain {
            my_cluster_name = &node_network_address.cluster;
            my_external_port = node_network_address.port;
            my_name_space = &node_network_address.name_space;
        }
    }

    if my_external_port == 0 {
        panic!("can't find domain in chain_config.node_network_address_list");
    }

    let is_k8s = !my_cluster_name.is_empty();
    if !is_k8s {
        panic!("current node is not k8s node");
    }

    let config_file_name = format!("{}/{}", &node_dir, opts.config_name);

    let yamls_path = format!("{}/yamls", &node_dir);
    fs::create_dir_all(&yamls_path).unwrap();

    // protocol now only support netowrk_zenoh which use quic
    let network_protocol = "UDP";

    // update yaml
    // node svc
    // statefulset
    // configmap about config.toml
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
            access_modes: Some(vec![opts.access_mode.clone()]),
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

        let mut template_spec = PodSpec {
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
                    mount_path: "/mnt".to_string(),
                    name: "node-account".to_string(),
                    ..Default::default()
                },
                VolumeMount {
                    mount_path: "/etc/localtime".to_string(),
                    name: "node-localtime".to_string(),
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
                            "grpc-health-probe".to_string(),
                            "-addr=localhost:50000".to_string(),
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
                    ]);
                } else {
                    panic!("Unknown network service!");
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
                    mount_path: "/mnt".to_string(),
                    name: "node-account".to_string(),
                    ..Default::default()
                },
                VolumeMount {
                    mount_path: "/etc/localtime".to_string(),
                    name: "node-localtime".to_string(),
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
                            "grpc-health-probe".to_string(),
                            "-addr=localhost:50001".to_string(),
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
                        "-p".to_string(),
                        "/mnt/private_key".to_string(),
                    ]);
                } else {
                    panic!("Unknown consensus service!");
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
                    mount_path: "/etc/localtime".to_string(),
                    name: "node-localtime".to_string(),
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
                            "grpc-health-probe".to_string(),
                            "-addr=localhost:50002".to_string(),
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
                    ]);
                } else {
                    panic!("Unknown executor service!");
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
                    mount_path: "/etc/localtime".to_string(),
                    name: "node-localtime".to_string(),
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
                            "grpc-health-probe".to_string(),
                            "-addr=localhost:50003".to_string(),
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
                if micro_service.image == STORAGE_OPENDAL {
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
                    ]);
                } else {
                    panic!("Unknown storage service!");
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
                    mount_path: "/mnt".to_string(),
                    name: "node-account".to_string(),
                    ..Default::default()
                },
                VolumeMount {
                    mount_path: "/etc/localtime".to_string(),
                    name: "node-localtime".to_string(),
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
                            "grpc-health-probe".to_string(),
                            "-addr=localhost:50004".to_string(),
                        ]),
                    }),
                    initial_delay_seconds: Some(60),
                    period_seconds: Some(10),
                    ..Default::default()
                })
            },
            resources: Some(container_resources_requirements),
            ..Default::default()
        };

        for micro_service in &chain_config.micro_service_list {
            if micro_service.image.starts_with("controller") {
                if micro_service.image == CONTROLLER || micro_service.image == CONTROLLER_HSM {
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
                        "-p".to_string(),
                        "/mnt/private_key".to_string(),
                    ]);
                } else {
                    panic!("Unknown controller service!");
                }
            }
        }

        containers.push(controller_container);

        // debug
        if opts.enable_debug {
            let mut debug_container = Container {
                name: "debug".to_string(),
                image_pull_policy: Some(opts.pull_policy.clone()),
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
                        mount_path: "/mnt".to_string(),
                        name: "node-account".to_string(),
                        ..Default::default()
                    },
                    VolumeMount {
                        mount_path: "/etc/localtime".to_string(),
                        name: "node-localtime".to_string(),
                        ..Default::default()
                    },
                ]),
                working_dir: Some("/data".to_string()),
                command: Some(vec![
                    "/bin/bash".to_string(),
                    "-c".to_string(),
                    "while true;do sleep 100;done".to_string(),
                ]),
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
                name: "node-localtime".to_string(),
                host_path: Some(HostPathVolumeSource {
                    path: "/etc/localtime".to_string(),
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
        match_labels.insert(
            "app.kubernetes.io/chain-name".to_string(),
            opts.chain_name.clone(),
        );
        match_labels.insert(
            "app.kubernetes.io/chain-node".to_string(),
            node_name.clone(),
        );
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

    {
        // create external svc and endpoints for peers
        // ignore if same cluster and same namespace
        // if same cluster and different namespace, create external svc with peer svc cluster FQDN
        // if diffrent cluster and peer host is FQDN, create external svc with peer host
        // if diffrent cluster and peer host is ip, create external svc map port to 40000 and create external endpoints
        for node_network_address in &chain_config.node_network_address_list {
            if node_network_address.domain != opts.domain {
                let peer_cluster_name = &node_network_address.cluster;
                let peer_name_space = &node_network_address.name_space;
                let peer_domain = &node_network_address.domain;
                let peer_host = &node_network_address.host;
                let is_peer_host_ip =
                    peer_host.parse::<Ipv4Addr>().is_ok() || peer_host.parse::<Ipv6Addr>().is_ok();
                let peer_port = node_network_address.port;
                let peer_svc_name =
                    format!("{}-{}", &opts.chain_name, &node_network_address.domain);

                let same_cluster = peer_cluster_name == my_cluster_name;
                let same_name_space = peer_name_space == my_name_space;
                match (same_cluster, same_name_space, is_peer_host_ip) {
                    (true, true, _) => {}
                    (true, false, _) => {
                        let mut external_svc = Service::default();

                        let metadata = ObjectMeta {
                            name: Some(peer_svc_name.clone()),
                            ..Default::default()
                        };
                        external_svc.metadata = metadata;

                        let svc_spec = ServiceSpec {
                            type_: Some("ExternalName".to_string()),
                            external_name: Some(format!(
                                "{}.{}.svc.cluster.local",
                                &peer_svc_name, &peer_name_space
                            )),
                            ..Default::default()
                        };
                        external_svc.spec = Some(svc_spec);

                        let yaml_file_name =
                            format!("{}/{}-external-svc.yaml", &yamls_path, &peer_domain);
                        write_file(
                            serde_yaml::to_string(&external_svc).unwrap().as_bytes(),
                            yaml_file_name,
                        );
                        node_k8s_config.external_svc.push(external_svc);
                    }
                    (false, _, false) => {
                        let mut external_svc = Service::default();

                        let metadata = ObjectMeta {
                            name: Some(peer_svc_name.clone()),
                            ..Default::default()
                        };
                        external_svc.metadata = metadata;

                        let svc_spec = ServiceSpec {
                            type_: Some("ExternalName".to_string()),
                            external_name: Some(peer_host.clone()),
                            ..Default::default()
                        };
                        external_svc.spec = Some(svc_spec);

                        let yaml_file_name =
                            format!("{}/{}-external-svc.yaml", &yamls_path, &peer_domain);
                        write_file(
                            serde_yaml::to_string(&external_svc).unwrap().as_bytes(),
                            yaml_file_name,
                        );
                        node_k8s_config.external_svc.push(external_svc);
                    }
                    (false, _, true) => {
                        let mut external_svc = Service::default();
                        let metadata = ObjectMeta {
                            name: Some(peer_svc_name.clone()),
                            ..Default::default()
                        };
                        external_svc.metadata = metadata;

                        let svc_spec = ServiceSpec {
                            ports: Some(vec![ServicePort {
                                name: Some("network".to_string()),
                                port: 40000,
                                target_port: Some(IntOrString::Int(peer_port as i32)),
                                protocol: Some(network_protocol.to_string()),
                                ..Default::default()
                            }]),
                            ..Default::default()
                        };
                        external_svc.spec = Some(svc_spec);

                        let mut endpoint_slice = EndpointSlice::default();
                        let mut metadata = ObjectMeta {
                            name: Some(format!("{}-1", peer_svc_name)),
                            ..Default::default()
                        };

                        let mut labels = BTreeMap::new();
                        labels.insert(
                            "kubernetes.io/service-name".to_string(),
                            peer_svc_name.clone(),
                        );
                        metadata.labels = Some(labels);

                        endpoint_slice.metadata = metadata;

                        if peer_host.parse::<Ipv4Addr>().is_ok() {
                            endpoint_slice.address_type = "IPv4".to_string();
                        } else {
                            endpoint_slice.address_type = "IPv6".to_string();
                        }

                        endpoint_slice.ports = Some(vec![EndpointPort {
                            port: Some(peer_port as i32),
                            protocol: Some(network_protocol.to_string()),
                            ..Default::default()
                        }]);

                        endpoint_slice.endpoints = vec![Endpoint {
                            addresses: vec![peer_host.clone()],
                            ..Default::default()
                        }];

                        let yaml_file_name =
                            format!("{}/{}-external-svc.yaml", &yamls_path, &peer_domain);
                        write_file(
                            serde_yaml::to_string(&external_svc).unwrap().as_bytes(),
                            yaml_file_name,
                        );

                        let yaml_file_name = format!(
                            "{}/{}-external-endpointslice.yaml",
                            &yamls_path, &peer_domain
                        );
                        write_file(
                            serde_yaml::to_string(&endpoint_slice).unwrap().as_bytes(),
                            yaml_file_name,
                        );

                        node_k8s_config.external_svc.push(external_svc);
                        node_k8s_config.external_endpoints.push(endpoint_slice);
                    }
                }
            }
        }
    }

    if opts.enable_kustomize {
        let kustomization = include_str!("../kustomization/kustomization.yaml");
        let yaml_file_name = format!("{}/kustomization.yaml", &node_dir);
        write_file(kustomization.as_bytes(), yaml_file_name);

        let patch_liveness_probe = include_str!("../kustomization/statefulset-livenessprobe.yaml");
        let yaml_file_name = format!("{}/statefulset-livenessprobe.yaml", &node_dir);
        write_file(
            patch_liveness_probe.replace("xxx", &node_name).as_bytes(),
            yaml_file_name,
        );

        let patch_pullpolicy = include_str!("../kustomization/statefulset-pullpolicy.yaml");
        let yaml_file_name = format!("{}/statefulset-pullpolicy.yaml", &node_dir);
        write_file(
            patch_pullpolicy.replace("xxx", &node_name).as_bytes(),
            yaml_file_name,
        );

        let patch_pvc = include_str!("../kustomization/statefulset-pvc.yaml");
        let yaml_file_name = format!("{}/statefulset-pvc.yaml", &node_dir);
        write_file(
            patch_pvc.replace("xxx", &node_name).as_bytes(),
            yaml_file_name,
        );

        let patch_resource = include_str!("../kustomization/statefulset-resource.yaml");
        let yaml_file_name = format!("{}/statefulset-resource.yaml", &node_dir);
        write_file(
            patch_resource.replace("xxx", &node_name).as_bytes(),
            yaml_file_name,
        );
    }

    Ok(node_k8s_config)
}
