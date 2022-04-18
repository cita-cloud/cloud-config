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
    CHAIN_CONFIG_FILE, CONSENSUS_BFT, CONSENSUS_RAFT, CONTROLLER, EXECUTOR_EVM, KMS_DB, KMS_ETH,
    KMS_SM, NETWORK_P2P, NETWORK_TLS, NODE_CONFIG_FILE, STORAGE_ROCKSDB,
};
use crate::error::Error;
use crate::util::{read_chain_config, read_file, read_node_config, svc_name, write_file};
use clap::Parser;
use k8s_openapi::api::apps::v1::StatefulSet;
use k8s_openapi::api::apps::v1::StatefulSetSpec;
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::api::core::v1::ConfigMapVolumeSource;
use k8s_openapi::api::core::v1::Container;
use k8s_openapi::api::core::v1::ContainerPort;
use k8s_openapi::api::core::v1::PersistentVolumeClaim;
use k8s_openapi::api::core::v1::PersistentVolumeClaimSpec;
use k8s_openapi::api::core::v1::PodSpec;
use k8s_openapi::api::core::v1::PodTemplateSpec;
use k8s_openapi::api::core::v1::ResourceRequirements;
use k8s_openapi::api::core::v1::Service;
use k8s_openapi::api::core::v1::ServicePort;
use k8s_openapi::api::core::v1::ServiceSpec;
use k8s_openapi::api::core::v1::Volume;
use k8s_openapi::api::core::v1::VolumeMount;
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use k8s_openapi::ByteString;
use std::collections::BTreeMap;
use std::fs;

/// A subcommand for run
#[derive(Parser, Debug, Clone)]
pub struct UpdateYamlOpts {
    /// set chain name
    #[clap(long = "chain-name", default_value = "test-chain")]
    pub(crate) chain_name: String,
    /// set config file directory, default means current directory
    #[clap(long = "config-dir", default_value = ".")]
    pub(crate) config_dir: String,
    /// set config file name
    #[clap(long = "config-name", default_value = "config.toml")]
    pub(crate) config_name: String,
    /// domain of node
    #[clap(long = "domain")]
    pub(crate) domain: String,
    /// image pull policy: IfNotPresent or Always
    #[clap(long = "pull-policy", default_value = "IfNotPresent")]
    pub(crate) pull_policy: String,
    /// docker registry
    #[clap(long = "docker-registry", default_value = "docker.io")]
    pub(crate) docker_registry: String,
    /// docker repo
    #[clap(long = "docker-repo", default_value = "citacloud")]
    pub(crate) docker_repo: String,
    /// storage class
    #[clap(long = "storage-class")]
    pub(crate) storage_class: String,
    /// storage capacity
    #[clap(long = "storage-capacity", default_value = "10Gi")]
    pub(crate) storage_capacity: String,
    /// is enable debug
    #[clap(long = "enable-debug")]
    pub(crate) enable_debug: bool,
}

/// generate k8s yaml by chain_config and node_config
pub fn execute_update_yaml(opts: UpdateYamlOpts) -> Result<(), Error> {
    let node_name = format!("{}-{}", &opts.chain_name, &opts.domain);
    let node_dir = format!("{}/{}", &opts.config_dir, &node_name);

    // load node_config
    let file_name = format!("{}/{}", &node_dir, NODE_CONFIG_FILE);
    let node_config = read_node_config(file_name).unwrap();

    // load chain_config
    let file_name = format!("{}/{}", &node_dir, CHAIN_CONFIG_FILE);
    let chain_config = read_chain_config(&file_name).unwrap();

    let config_file_name = format!("{}/{}", &node_dir, opts.config_name);

    let yamls_path = format!("{}/yamls", &node_dir);
    fs::create_dir_all(&yamls_path).unwrap();

    // update yaml
    // node port svc
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
            read_file(&config_file_name).unwrap(),
        );
        cm_config.data = Some(data);

        let yaml_file_name = format!("{}/{}-cm-config.yaml", &yamls_path, &node_name);
        write_file(
            serde_yaml::to_string(&cm_config).unwrap().as_bytes(),
            &yaml_file_name,
        );
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
            read_file(&format!("{}/consensus-log4rs.yaml", &node_dir)).unwrap(),
        );
        data.insert(
            "controller-log4rs.yaml".to_string(),
            read_file(&format!("{}/controller-log4rs.yaml", &node_dir)).unwrap(),
        );
        data.insert(
            "executor-log4rs.yaml".to_string(),
            read_file(&format!("{}/executor-log4rs.yaml", &node_dir)).unwrap(),
        );
        data.insert(
            "kms-log4rs.yaml".to_string(),
            read_file(&format!("{}/kms-log4rs.yaml", &node_dir)).unwrap(),
        );
        data.insert(
            "network-log4rs.yaml".to_string(),
            read_file(&format!("{}/network-log4rs.yaml", &node_dir)).unwrap(),
        );
        data.insert(
            "storage-log4rs.yaml".to_string(),
            read_file(&format!("{}/storage-log4rs.yaml", &node_dir)).unwrap(),
        );
        cm_log.data = Some(data);

        let yaml_file_name = format!("{}/{}-cm-log.yaml", &yamls_path, &node_name);
        write_file(
            serde_yaml::to_string(&cm_log).unwrap().as_bytes(),
            &yaml_file_name,
        );
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
        data.insert("address".to_string(), node_config.account.clone());
        data.insert("keyId".to_string(), format!("{}", node_config.key_id));
        cm_account.data = Some(data);

        let mut binary_data = BTreeMap::new();
        binary_data.insert(
            KMS_DB.to_string(),
            ByteString(fs::read(&format!("{}/{}", &node_dir, KMS_DB)).unwrap()),
        );
        cm_account.binary_data = Some(binary_data);

        let yaml_file_name = format!("{}/{}-cm-account.yaml", &yamls_path, &node_name);
        write_file(
            serde_yaml::to_string(&cm_account).unwrap().as_bytes(),
            &yaml_file_name,
        );
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

        let mut template_spec = PodSpec::default();
        let mut containers = Vec::new();
        // network
        let mut network_container = Container {
            name: "network".to_string(),
            image_pull_policy: Some(opts.pull_policy.clone()),
            ports: Some(vec![
                ContainerPort {
                    container_port: 40000,
                    name: Some("network".to_string()),
                    protocol: Some("TCP".to_string()),
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
            ]),
            working_dir: Some("/data".to_string()),
            ..Default::default()
        };

        for micro_service in &chain_config.micro_service_list {
            if micro_service.image == NETWORK_TLS {
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
                    "--stdout".to_string(),
                ]);
            } else if micro_service.image == NETWORK_P2P {
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
            ]),
            working_dir: Some("/data".to_string()),
            ..Default::default()
        };

        for micro_service in &chain_config.micro_service_list {
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
            } else if micro_service.image == CONSENSUS_BFT {
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
                ]);
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
            ..Default::default()
        };

        for micro_service in &chain_config.micro_service_list {
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
            ..Default::default()
        };

        for micro_service in &chain_config.micro_service_list {
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
            ]),
            working_dir: Some("/data".to_string()),
            ..Default::default()
        };

        for micro_service in &chain_config.micro_service_list {
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
            }
        }

        containers.push(controller_container);

        // kms
        let mut kms_container = Container {
            name: "kms".to_string(),
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
            ..Default::default()
        };

        for micro_service in &chain_config.micro_service_list {
            if micro_service.image == KMS_ETH || micro_service.image == KMS_SM {
                kms_container.image = Some(format!(
                    "{}/{}/{}:{}",
                    &opts.docker_registry,
                    &opts.docker_repo,
                    &micro_service.image,
                    &micro_service.tag
                ));
                kms_container.command = Some(vec!["/bin/sh".to_string(),
          "-c".to_string(),
          "if [ ! -f \"/data/kms.db\" ]; then cp /mnt/kms.db /data;fi; kms run -c /etc/cita-cloud/config/config.toml -l /etc/cita-cloud/log/kms-log4rs.yaml".to_string(),
          ]);
            }
        }

        containers.push(kms_container);

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

        let yaml_file_name = format!("{}/{}.yaml", &yamls_path, &node_name);
        write_file(
            serde_yaml::to_string(&statefulset).unwrap().as_bytes(),
            &yaml_file_name,
        );
    }

    {
        let mut nodeport_svc = Service::default();

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

        nodeport_svc.metadata = metadata;

        let mut svc_spec = ServiceSpec {
            type_: Some("NodePort".to_string()),
            ..Default::default()
        };

        let mut match_labels = BTreeMap::new();
        match_labels.insert("app.kubernetes.io/chain-name".to_string(), opts.chain_name);
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
                protocol: Some("TCP".to_string()),
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

        nodeport_svc.spec = Some(svc_spec);

        let yaml_file_name = format!("{}/{}-svc.yaml", &yamls_path, &node_name);
        write_file(
            serde_yaml::to_string(&nodeport_svc).unwrap().as_bytes(),
            &yaml_file_name,
        );
    }

    Ok(())
}
