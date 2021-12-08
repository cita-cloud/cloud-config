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

// This is from [migration-tool](https://github.com/cita-cloud/migration-tool)

// cert.rs
mod cert {
    use rcgen::BasicConstraints;
    use rcgen::Certificate;
    use rcgen::CertificateParams;
    use rcgen::IsCa;
    use rcgen::KeyPair;
    use rcgen::PKCS_ECDSA_P256_SHA256;

    pub struct CertAndKey {
        pub cert: String,
        pub key: String,
    }

    fn ca_cert() -> (Certificate, CertAndKey) {
        let mut params = CertificateParams::new(vec![]);
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);

        let keypair = KeyPair::generate(&PKCS_ECDSA_P256_SHA256).unwrap();
        params.key_pair.replace(keypair);

        let cert = Certificate::from_params(params).unwrap();
        let cert_and_key = {
            let cert_pem = cert.serialize_pem_with_signer(&cert).unwrap();
            let key_pem = cert.serialize_private_key_pem();
            CertAndKey {
                cert: cert_pem,
                key: key_pem,
            }
        };

        (cert, cert_and_key)
    }

    fn cert(domain: &str, signer: &Certificate) -> (Certificate, CertAndKey) {
        let subject_alt_names = vec![domain.into()];
        let mut params = CertificateParams::new(subject_alt_names);

        let keypair = KeyPair::generate(&PKCS_ECDSA_P256_SHA256).unwrap();
        params.key_pair.replace(keypair);

        let cert = Certificate::from_params(params).unwrap();
        let cert_and_key = {
            let cert_pem = cert.serialize_pem_with_signer(signer).unwrap();
            let key_pem = cert.serialize_private_key_pem();
            CertAndKey {
                cert: cert_pem,
                key: key_pem,
            }
        };
        (cert, cert_and_key)
    }

    pub fn generate_certs(domains: &[String]) -> (CertAndKey, Vec<CertAndKey>) {
        let (ca_cert, ca_cert_and_key) = ca_cert();
        let peer_cert_and_keys = domains
            .iter()
            .map(|domain| cert(domain, &ca_cert).1)
            .collect();

        (ca_cert_and_key, peer_cert_and_keys)
    }
}

// migrate.rs
mod migrate_impl {
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::fs;
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::Path;
    use std::path::PathBuf;

    use fs_extra::dir::copy as copy_dir;
    use fs_extra::dir::CopyOptions;

    use anyhow::ensure;
    use anyhow::Context;
    use anyhow::Result;
    use serde::de::DeserializeOwned;

    use super::cert::{generate_certs, CertAndKey};

    pub use new::ConsensusType;

    mod old {
        use serde::Deserialize;

        #[derive(Deserialize)]
        pub struct ConsensusConfig {
            pub controller_port: u16,
        }

        #[derive(Deserialize)]
        pub struct ControllerConfig {
            pub network_port: u16,
            pub consensus_port: u16,
            pub storage_port: u16,
            pub kms_port: u16,
            pub executor_port: u16,
        }

        #[derive(Deserialize)]
        pub struct NetworkConfig {
            pub port: u16,
            pub peers: Vec<PeerConfig>,
        }

        #[derive(Deserialize)]
        pub struct PeerConfig {
            pub ip: String,
            pub port: u16,
        }

        #[derive(Deserialize)]
        pub struct InitSysConfig {
            pub version: u64,
            pub admin: String,
            pub block_interval: u64,
            pub chain_id: String,
            pub validators: Vec<String>,
        }

        #[derive(Deserialize)]
        pub struct Genesis {
            pub timestamp: u64,
            pub prevhash: String,
        }
    }

    mod new {
        use serde::Serialize;

        pub const DEFAULT_BLOCK_LIMIT: u64 = 100;
        pub const DEFAULT_PACKAGE_LIMIT: u64 = 30000;

        #[derive(Serialize)]
        pub struct ControllerConfig {
            pub consensus_port: u16,
            pub controller_port: u16,
            pub executor_port: u16,
            pub storage_port: u16,
            pub kms_port: u16,
            pub network_port: u16,

            // key_id will be filled later when kms.db is generated.
            pub key_id: Option<u64>,
            pub node_address: String,
            pub package_limit: u64,
        }

        #[derive(Serialize)]
        pub struct ConsensusRaftConfig {
            pub controller_port: u16,
            pub grpc_listen_port: u16,
            pub network_port: u16,
            pub node_addr: String,
        }

        #[derive(Serialize)]
        pub struct ConsensusBftConfig {
            pub controller_port: u16,
            pub consensus_port: u16,
            pub network_port: u16,
            pub kms_port: u16,
            pub node_address: String,
        }

        #[derive(Serialize)]
        pub enum ConsensusConfig {
            #[serde(rename = "consensus_raft")]
            Raft(ConsensusRaftConfig),
            #[serde(rename = "consensus_bft")]
            Bft(ConsensusBftConfig),
        }

        #[derive(Clone, Copy)]
        pub enum ConsensusType {
            Raft,
            Bft,
        }

        #[derive(Serialize, Clone)]
        pub struct GenesisBlock {
            pub prevhash: String,
            pub timestamp: u64,
        }

        #[derive(Serialize, Clone)]
        pub struct SystemConfig {
            pub admin: String,
            pub block_interval: u64,
            pub block_limit: u64,
            pub chain_id: String,
            pub version: u64,
            pub validators: Vec<String>,
        }

        #[derive(Serialize)]
        pub struct NetworkTlsConfig {
            // Optional fields will be filled latter
            pub ca_cert: Option<String>,
            pub cert: Option<String>,
            pub priv_key: Option<String>,
            pub grpc_port: u16,
            pub listen_port: u16,
            pub peers: Vec<NetworkTlsPeerConfig>,
        }

        #[derive(Serialize, Clone)]
        pub struct NetworkTlsPeerConfig {
            // Will be filled latter
            pub domain: Option<String>,
            pub host: String,
            pub port: u16,
        }

        #[derive(Serialize)]
        pub struct KmsSmConfig {
            pub kms_port: u16,
            pub db_key: String,
        }

        #[derive(Serialize)]
        pub struct StorageRocksDbConfig {
            pub kms_port: u16,
            pub storage_port: u16,
        }

        #[derive(Serialize)]
        pub struct ExecutorEvmConfig {
            pub executor_port: u16,
        }

        #[derive(Serialize)]
        pub struct Config {
            pub system_config: SystemConfig,
            pub genesis_block: GenesisBlock,

            #[serde(rename = "controller")]
            pub controller: ControllerConfig,
            #[serde(flatten)]
            pub consensus: ConsensusConfig,
            #[serde(rename = "storage_rocksdb")]
            pub storage: StorageRocksDbConfig,
            #[serde(rename = "executor_evm")]
            pub executor: ExecutorEvmConfig,
            #[serde(rename = "kms_sm")]
            pub kms: KmsSmConfig,
            #[serde(rename = "network_tls")]
            pub network: NetworkTlsConfig,

            // Helper data, Option will be filled later
            #[serde(skip)]
            pub node_key: String,
            #[serde(skip)]
            pub network_host: Option<String>,
            #[serde(skip)]
            pub network_port: Option<u16>,
        }

        #[derive(Serialize)]
        pub struct MetaConfig {
            #[serde(rename = "network_tls")]
            pub network: MetaNetworkConfig,

            pub genesis_block: GenesisBlock,
            pub system_config: SystemConfig,

            pub admin_config: MetaAdminConfig,

            pub current_config: MetaCurrentConfig,
        }

        #[derive(Serialize)]
        pub struct MetaAdminConfig {
            pub admin_address: String,
            // pub key_id: u64,
        }

        #[derive(Serialize)]
        pub struct MetaCurrentConfig {
            pub addresses: Vec<String>,

            pub ca_cert_pem: String,
            pub ca_key_pem: String,

            pub count: u64,

            pub ips: Vec<String>,
            pub p2p_ports: Vec<u16>,
            pub rpc_ports: Vec<u16>,

            // Always false
            pub use_num: bool,

            pub tls_peers: MetaNetworkConfig,
        }

        #[derive(Serialize, Clone)]
        pub struct MetaNetworkConfig {
            pub peers: Vec<NetworkTlsPeerConfig>,
        }
    }

    struct NodeConfigMigrate {
        // node config loaded from old

        // ports
        controller_port: u16,
        consensus_port: u16,
        executor_port: u16,
        network_port: u16,
        kms_port: u16,
        storage_port: u16,

        // controller
        node_addr: String,
        genesis_block: old::Genesis,
        system_config: old::InitSysConfig,

        // kms
        kms_password: String,
        node_key: String,

        // network
        network_config: old::NetworkConfig,
    }

    impl NodeConfigMigrate {
        pub fn from_old(
            data_dir: impl AsRef<Path>,
            consensus_type: new::ConsensusType,
        ) -> Result<new::Config> {
            let old =
                Self::extract_from(data_dir).context("cannot extract info from old node config")?;
            Ok(old.generate_new(consensus_type))
        }

        fn extract_from(data_dir: impl AsRef<Path>) -> Result<Self> {
            let old::ControllerConfig {
                consensus_port,
                storage_port,
                network_port,
                executor_port,
                kms_port,
            } = extract_toml(&data_dir, "controller-config.toml")?;

            let old::ConsensusConfig { controller_port } =
                extract_toml(&data_dir, "consensus-config.toml")?;

            let network_config: old::NetworkConfig =
                extract_toml(&data_dir, "network-config.toml")?;
            let node_addr = extract_text(&data_dir, "node_address")?;

            let system_config: old::InitSysConfig =
                extract_toml(&data_dir, "init_sys_config.toml")?;
            let genesis_block: old::Genesis = extract_toml(&data_dir, "genesis.toml")?;

            let node_key = extract_text(&data_dir, "node_key")?;
            let kms_password = {
                use rand::Rng;
                let random: [u8; 32] = rand::thread_rng().gen();
                hex::encode(&random)
            };

            let this = Self {
                controller_port,
                consensus_port,
                executor_port,
                network_port,
                kms_port,
                storage_port,

                // controller
                node_addr,
                genesis_block,
                system_config,

                // kms
                kms_password,
                node_key,

                // network
                network_config,
            };

            Ok(this)
        }

        fn generate_new(&self, consensus_type: new::ConsensusType) -> new::Config {
            let genesis_block = new::GenesisBlock {
                prevhash: self.genesis_block.prevhash.clone(),
                timestamp: self.genesis_block.timestamp,
            };

            let system_config = new::SystemConfig {
                admin: self.system_config.admin.clone(),
                block_interval: self.system_config.block_interval,
                block_limit: new::DEFAULT_BLOCK_LIMIT,
                chain_id: self.system_config.chain_id.clone(),
                validators: self.system_config.validators.clone(),
                version: self.system_config.version,
            };

            let controller = new::ControllerConfig {
                consensus_port: self.consensus_port,
                controller_port: self.controller_port,
                executor_port: self.executor_port,
                network_port: self.network_port,
                kms_port: self.kms_port,
                storage_port: self.storage_port,

                // will be filled later
                key_id: None,
                node_address: self.node_addr.clone(),
                package_limit: new::DEFAULT_PACKAGE_LIMIT,
            };

            let consensus = match consensus_type {
                new::ConsensusType::Raft => new::ConsensusConfig::Raft(new::ConsensusRaftConfig {
                    controller_port: self.controller_port,
                    network_port: self.network_port,
                    node_addr: self.node_addr.clone(),
                    grpc_listen_port: self.consensus_port,
                }),
                new::ConsensusType::Bft => new::ConsensusConfig::Bft(new::ConsensusBftConfig {
                    controller_port: self.controller_port,
                    network_port: self.network_port,
                    consensus_port: self.consensus_port,
                    kms_port: self.kms_port,
                    node_address: self.node_addr.clone(),
                }),
            };

            let kms = new::KmsSmConfig {
                kms_port: self.kms_port,
                db_key: self.kms_password.clone(),
            };

            let storage = new::StorageRocksDbConfig {
                kms_port: self.kms_port,
                storage_port: self.storage_port,
            };

            let executor = new::ExecutorEvmConfig {
                executor_port: self.executor_port,
            };

            let network = {
                let peers = self
                    .network_config
                    .peers
                    .iter()
                    .map(|p| {
                        new::NetworkTlsPeerConfig {
                            // will be filled latter
                            domain: None,
                            host: p.ip.clone(),
                            port: p.port,
                        }
                    })
                    .collect();

                new::NetworkTlsConfig {
                    // will be filled latter
                    ca_cert: None,
                    cert: None,
                    priv_key: None,
                    grpc_port: self.network_port,
                    // listen network peers' connections
                    listen_port: self.network_config.port,
                    peers,
                }
            };

            new::Config {
                system_config,
                genesis_block,

                controller,
                consensus,
                executor,
                storage,
                kms,
                network,

                node_key: self.node_key.clone(),
                network_host: None,
                network_port: None,
            }
        }
    }

    fn extract_toml<T: DeserializeOwned>(data_dir: impl AsRef<Path>, file_name: &str) -> Result<T> {
        let s = extract_text(data_dir, file_name).context("cannot load toml file")?;
        let res: T = toml::from_str(&s).with_context(|| {
            format!("invalid toml for the `{}` type", std::any::type_name::<T>())
        })?;
        Ok(res)
    }

    fn extract_text(data_dir: impl AsRef<Path>, file_name: &str) -> Result<String> {
        let path = data_dir.as_ref().join(file_name);
        let mut f = File::open(&path).with_context(|| {
            format!(
                "cannot open file `{}` in `{}`",
                file_name,
                data_dir.as_ref().display()
            )
        })?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)
            .with_context(|| format!("cannot read data from {}", path.display()))?;
        Ok(buf)
    }

    // Return CA's cert and key
    fn fill_network_tls_info(node_configs: &mut [new::Config]) -> Result<CertAndKey> {
        // Construct (host, port) -> node_addr map.
        let host_port_to_addr: HashMap<(String, u16), String> = {
            let full_peer_set = {
                let mut full_peer_set = HashSet::<(String, u16)>::new();
                // Every node contains host and port for peers execept itself.
                // So we can construct the full set with two configs.
                for c in node_configs.iter().take(2) {
                    for p in &c.network.peers {
                        full_peer_set.insert((p.host.clone(), p.port));
                    }
                }
                full_peer_set
            };

            if full_peer_set.is_empty() {
                let c = node_configs
                    .first_mut()
                    .context("Empty chain. No node config found.")?;
                let host = String::from("localhost");
                let port = c.network.listen_port;
                c.network_host.replace(host.clone());
                c.network_port.replace(port);

                // It isn't necessary and won't be used since the single node's network config contains no other peers.
                let mut single_node_map = HashMap::new();
                single_node_map.insert((host, port), c.controller.node_address.clone());
                single_node_map
            } else {
                // Find nodes' host and port
                node_configs
                    .iter_mut()
                    .map(|c| {
                        let peer_set: HashSet<(String, u16)> = c
                            .network
                            .peers
                            .iter()
                            .map(|p| (p.host.clone(), p.port))
                            .collect();
                        let (host, port) = full_peer_set.difference(&peer_set).next().cloned()
                            .context(
                                "Cannot find out node's self host and port. \
                                The assumption that node's peers info contains all (and only) other peers has been violated"
                            )?;
                        c.network_host.replace(host.clone());
                        c.network_port.replace(port);

                        Ok(
                            ((host, port), c.controller.node_address.clone())
                        )
                    })
                    .collect::<Result<_>>()?
            }
        };

        let node_addrs: Vec<String> = node_configs
            .iter()
            .map(|c| c.controller.node_address.clone())
            .collect();
        let (ca_cert_and_key, peer_cert_and_keys) = generate_certs(&node_addrs);

        node_configs
            .iter_mut()
            .zip(peer_cert_and_keys)
            .try_for_each(|(c, cert_and_key)| {
                c.network.ca_cert.replace(ca_cert_and_key.cert.clone());
                c.network.cert.replace(cert_and_key.cert);
                c.network.priv_key.replace(cert_and_key.key);

                for p in c.network.peers.iter_mut() {
                    let node_addr = host_port_to_addr
                        .get(&(p.host.clone(), p.port))
                        .cloned()
                        .with_context(|| {
                            format!(
                                "cannot find node address for `{}:{}`. go check network config",
                                &p.host, p.port
                            )
                        })?;
                    p.domain.replace(node_addr);
                }
                Ok::<(), anyhow::Error>(())
            })?;

        Ok(ca_cert_and_key)
    }

    pub fn migrate<P, Q>(
        chain_data_dir: P,
        new_chain_data_dir: Q,
        chain_name: &str,
        consensus_type: new::ConsensusType,
    ) -> Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        let chain_data_dir = chain_data_dir.as_ref();
        ensure!(chain_data_dir.is_dir(), "chain data folder not found");

        let new_chain_data_dir = new_chain_data_dir.as_ref();
        let new_chain_metadata_dir = new_chain_data_dir.join(chain_name);

        // Load node dirs.
        let mut node_dirs: Vec<PathBuf> = fs::read_dir(chain_data_dir)
            .unwrap()
            .filter_map(|ent| {
                let ent = ent.unwrap();
                let dir_name = ent.file_name().into_string().unwrap();
                let prefix = format!("{}-", chain_name);
                if ent.file_type().unwrap().is_dir()
                    && dir_name.starts_with(&prefix)
                    && dir_name
                        .strip_prefix(&prefix)
                        .unwrap()
                        .parse::<u64>()
                        .is_ok()
                {
                    Some(ent.path())
                } else {
                    None
                }
            })
            .collect();

        // Sort node dirs according to their node_id.
        node_dirs.sort_by_key(|d| {
            let dir_name = d.file_name().unwrap().to_string_lossy();
            let node_id: u64 = dir_name
                .strip_prefix(&format!("{}-", chain_name))
                .unwrap()
                .parse()
                .unwrap();
            node_id
        });

        // Construct new node config from the old one. (without network_tls info)
        let mut node_configs = node_dirs
            .iter()
            .map(|d| {
                NodeConfigMigrate::from_old(d, consensus_type)
                    .with_context(|| format!("cannot migrate node config in `{}`", d.display()))
            })
            .collect::<Result<Vec<new::Config>>>()?;

        // Fill the network_tls info.
        let CertAndKey {
            cert: ca_cert_pem,
            key: ca_key_pem,
        } = fill_network_tls_info(&mut node_configs)
            .context("cannot fill network_tls info for chain config")?;

        // Construct $NEW_CHAIN_DATA_DIR/$CHAIN_NAME/config.toml
        let meta_config = {
            let node_addrs: Vec<String> = node_configs
                .iter()
                .map(|c| c.controller.node_address.clone())
                .collect();
            // Sample node
            let first_node = node_configs
                .first()
                .context("Empty chain. No node config found")?;
            let system_config = first_node.system_config.clone();
            let genesis_block = first_node.genesis_block.clone();

            let network_config = {
                let itself = new::NetworkTlsPeerConfig {
                    domain: Some(first_node.controller.node_address.clone()),
                    // Network info has been filled.
                    host: first_node.network_host.clone().unwrap(),
                    port: first_node.network_port.unwrap(),
                };
                let peers: Vec<new::NetworkTlsPeerConfig> = std::iter::once(itself)
                    .chain(first_node.network.peers.clone())
                    .collect();

                new::MetaNetworkConfig { peers }
            };

            let current_config = {
                let (ips, p2p_ports) = network_config
                    .peers
                    .iter()
                    .map(|p| (p.host.clone(), p.port))
                    .unzip();

                let rpc_ports = node_configs
                    .iter()
                    .map(|c| c.controller.controller_port)
                    .collect();

                new::MetaCurrentConfig {
                    addresses: node_addrs,
                    ca_cert_pem,
                    ca_key_pem,
                    count: node_configs.len() as u64,

                    ips,
                    p2p_ports,
                    rpc_ports,

                    use_num: false,
                    tls_peers: network_config.clone(),
                }
            };

            let admin_config = {
                let admin_address = first_node.system_config.admin.clone();
                new::MetaAdminConfig { admin_address }
            };

            new::MetaConfig {
                network: network_config,
                genesis_block,
                system_config,
                admin_config,
                current_config,
            }
        };

        fs::create_dir_all(&new_chain_data_dir).unwrap();
        fs::create_dir_all(&new_chain_metadata_dir).unwrap();

        // construct new meta data
        let mut meta_config_toml = File::create(new_chain_metadata_dir.join("config.toml"))
            .context("cannot create meta `config.toml`")?;
        let meta_config_content = toml::to_string_pretty(&meta_config).unwrap();
        meta_config_toml
            .write_all(meta_config_content.as_bytes())
            .context("cannot write meta `config.toml`")?;

        let sample_node = node_dirs.first().unwrap();
        migrate_log4rs(sample_node, new_chain_metadata_dir)
            .context("cannot copy log4rs to meta config dir")?;

        // construct new node data
        for (i, (old_node_dir, mut node_config)) in node_dirs.iter().zip(node_configs).enumerate() {
            let new_node_dir = new_chain_data_dir.join(format!("{}-{}", chain_name, i));
            fs::create_dir_all(&new_node_dir).with_context(|| {
                format!("cannot create new node dir `{}`", new_node_dir.display())
            })?;

            let key_id = generate_kms_db(
                &new_node_dir,
                &node_config.kms.db_key,
                &node_config.node_key,
            )
            .context("cannot generate kms db")?;
            node_config.controller.key_id.replace(key_id);

            let mut node_config_toml = File::create(new_node_dir.join("config.toml"))
                .context("cannot create node's `config.toml`")?;
            let node_config_content = toml::to_string_pretty(&node_config).unwrap();
            node_config_toml
                .write_all(node_config_content.as_bytes())
                .context("cannot write node's `config.toml`")?;

            migrate_log4rs(&old_node_dir, &new_node_dir).with_context(|| {
                format!(
                    "cannot migrate log4rs yamls for `{}`",
                    old_node_dir.display()
                )
            })?;
            migrate_runtime_data(&old_node_dir, &new_node_dir).with_context(|| {
                format!(
                    "cannot migrate runtime data for `{}`",
                    old_node_dir.display()
                )
            })?;
        }

        Ok(())
    }

    // return key_id of the account
    fn generate_kms_db(
        new_dir: impl AsRef<Path>,
        kms_password: &str,
        node_key: &str,
    ) -> Result<u64> {
        let db_path = new_dir.as_ref().join("kms.db").to_string_lossy().into();
        let priv_key = {
            let s = crate::util::remove_0x(node_key);
            hex::decode(s).context("invalid `node_key`")?
        };

        use crate::traits::Kms;
        let kms = crate::config::kms_sm::Kms::create_kms_db(db_path, kms_password.into());
        let key_id = kms.insert_privkey(priv_key);

        Ok(key_id)
    }

    fn migrate_log4rs<P, Q>(old_dir: P, new_dir: Q) -> Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        let old_dir = old_dir.as_ref();
        let new_dir = new_dir.as_ref();

        if new_dir == old_dir {
            return Ok(());
        }

        let files = [
            "controller-log4rs.yaml",
            "storage-log4rs.yaml",
            "executor-log4rs.yaml",
            "kms-log4rs.yaml",
            "network-log4rs.yaml",
            "consensus-log4rs.yaml",
        ];

        for f in files {
            let from = old_dir.join(f);
            let to = new_dir.join(f);
            fs::copy(&from, &to).with_context(|| {
                format!(
                    "cannot copy file from `{}` to `{}`",
                    from.display(),
                    to.display()
                )
            })?;
        }
        Ok(())
    }

    fn migrate_runtime_data<P, Q>(old_dir: P, new_dir: Q) -> Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        let old_dir = old_dir.as_ref();
        let new_dir = new_dir.as_ref();

        if new_dir == old_dir {
            return Ok(());
        }

        let dirs = ["chain_data", "data", "logs"];

        let opts = CopyOptions {
            skip_exist: true,
            copy_inside: true,
            ..Default::default()
        };
        for d in dirs {
            let from = old_dir.join(d);
            let to = new_dir.join(d);

            if from.exists() {
                copy_dir(&from, &to, &opts).with_context(|| {
                    format!(
                        "cannot copy dir from `{}` to `{}`",
                        from.display(),
                        to.display()
                    )
                })?;
            } else {
                println!(
                    "skip migration for `{}` since it doesn't exist.",
                    from.display()
                );
            }
        }

        Ok(())
    }
}

// main.rs

use clap::{Clap, ValueHint};

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;

use migrate_impl::{migrate, ConsensusType};

/// Migrate CITA-Cloud chain from 6.1.0 to 6.3.0
/// WARNING:
/// This is for a very specific use case, other cases may not work!
/// DO NOT use it if you don't know what is it for.
/// Back up your data before use it.
#[derive(Clap, Debug, Clone)]
pub struct MigrateOpts {
    /// The old chain dir
    #[clap(short = 'd', long = "chain-dir", value_hint = ValueHint::DirPath)]
    pub chain_dir: String,
    /// The output dir for the upgraded chain
    #[clap(short = 'o', long = "out-dir", value_hint = ValueHint::DirPath)]
    pub out_dir: String,
    /// Name of the chain
    #[clap(short = 'n', long = "chain-name")]
    pub chain_name: String,
    /// Consensus type, only `raft` or `bft` is supported
    #[clap(short = 'c', long = "consensus-type", default_value = "raft")]
    pub consensus_type: String,
}

pub fn execute_migrate(opts: MigrateOpts) -> Result<()> {
    let chain_dir = opts.chain_dir;
    let out_dir = opts.out_dir;
    let chain_name = opts.chain_name;
    let consensus_type = match opts.consensus_type.to_ascii_lowercase().as_str() {
        "raft" => ConsensusType::Raft,
        "bft" => ConsensusType::Bft,
        _ => bail!("unkown consensus type, possible values are [`raft`, `bft`]"),
    };

    migrate(&chain_dir, &out_dir, &chain_name, consensus_type).context("cannot migrate chain")?;

    println!("Finshed. new config write to `{}`", out_dir);
    Ok(())
}
