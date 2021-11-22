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

use crate::config::chain_config::ChainConfig;
use crate::config::node_config::NodeConfig;
use crate::traits::{AggregateConfig, Kms, TomlWriter};
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, IsCa, KeyPair, PKCS_ECDSA_P256_SHA256,
};
use regex::Regex;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, path};
use toml::de::Error;
use toml::Value;

pub fn write_to_file<T: serde::Serialize>(content: T, path: impl AsRef<path::Path>, name: String) {
    let value = Value::try_from(content).unwrap();
    let mut table = toml::map::Map::new();
    table.insert(name, value);
    let toml = toml::Value::Table(table);

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path.as_ref())
        .expect(&format!("open file({:?}) failed.", path.as_ref().to_str()));
    file.write_all(toml::to_string_pretty(&toml).unwrap().as_bytes())
        .unwrap();
    file.write_all(b"\n").unwrap();
}

pub fn write_whole_to_file(content: AggregateConfig, path: impl AsRef<path::Path>) {
    if let Some(t) = content.controller {
        t.write(&path);
    }
    if let Some(t) = content.kms_sm {
        t.write(&path);
    }
    if let Some(t) = content.storage_rocksdb {
        t.write(&path);
    }
    if let Some(t) = content.executor_evm {
        t.write(&path);
    }
    if let Some(t) = content.current_config {
        t.write(&path);
    }
    if let Some(t) = content.admin_config {
        t.write(&path);
    }
    if let Some(t) = content.consensus_raft {
        t.write(&path);
    }
    content.system_config.write(&path);
    content.genesis_block.write(&path);
    if let Some(t) = content.network_p2p {
        t.write(&path);
    }

    if let Some(t) = content.network_tls {
        t.write(&path);
    }
}

pub fn read_from_file(path: impl AsRef<path::Path>) -> Result<AggregateConfig, Error> {
    let buffer = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("Error while loading config: [{}]", err));
    toml::from_str::<AggregateConfig>(&buffer)
}

pub fn read_chain_config(path: impl AsRef<path::Path>) -> Result<ChainConfig, Error> {
    let buffer = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("Error while loading config: [{}]", err));
    toml::from_str::<ChainConfig>(&buffer)
}

pub fn read_node_config(path: impl AsRef<path::Path>) -> Result<NodeConfig, Error> {
    let buffer = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("Error while loading config: [{}]", err));
    toml::from_str::<NodeConfig>(&buffer)
}

pub fn write_toml<T: serde::Serialize>(content: T, path: impl AsRef<path::Path>) {
    let toml = Value::try_from(content).unwrap();

    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(path.as_ref())
        .expect(&format!("open file({:?}) failed.", path.as_ref().to_str()));
    file.write_all(toml::to_string_pretty(&toml).unwrap().as_bytes())
        .unwrap();
    file.write_all(b"\n").unwrap();
}

pub fn unix_now() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    let ms = since_the_epoch.as_secs() as i64 * 1000i64
        + (since_the_epoch.subsec_nanos() as i64 / 1_000_000) as i64;
    ms as u64
}

const HASH_BYTES_LEN: usize = 32;

pub fn sm3_hash(input: &[u8]) -> [u8; HASH_BYTES_LEN] {
    let mut result = [0u8; HASH_BYTES_LEN];
    result.copy_from_slice(libsm::sm3::hash::Sm3Hash::new(input).get_hash().as_ref());
    result
}

#[allow(dead_code)]
pub fn validate_p2p_ports(s: String) -> bool {
    match s {
        s if s.is_empty() => false,
        s => {
            let r = Regex::new(r"(^(\d|[1-9]\d|1\d{2}|2[0-4]\d|25[0-5])\.(\d|[1-9]\d|1\d{2}|2[0-4]\d|25[0-5])\.(\d|[1-9]\d|1\d{2}|2[0-4]\d|25[0-5])\.(\d|[1-9]\d|1\d{2}|2[0-4]\d|25[0-5]):([0-9]|[1-9]\d|[1-9]\d{2}|[1-9]\d{3}|[1-5]\d{4}|6[0-4]\d{3}|65[0-4]\d{2}|655[0-2]\d|6553[0-5])$)").unwrap();
            for item in s.split(',') {
                if !r.is_match(item) {
                    return false;
                }
            }
            true
        }
    }
}

pub fn key_pair(node_dir: String, kms_password: String) -> (u64, Vec<u8>) {
    let kms = crate::config::kms_sm::Kms::create_kms_db(
        format!("{}/{}", node_dir, "kms.db"),
        kms_password,
    );
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

pub fn clean_0x(s: &str) -> &str {
    if s.starts_with("0x") {
        &s[2..]
    } else {
        s
    }
}

#[cfg(test)]
mod util_test {
    use crate::util::read_from_file;
    use rand::prelude::*;
    use rcgen::{KeyPair, PKCS_ECDSA_P256_SHA256};

    // type Type = [u8, 32]

    #[test]
    fn util_test() {
        let config = read_from_file("cita-chain/config.toml");
        println!("{:?}", config)
    }

    #[test]
    fn random_address() {
        let rand: [u8; 16] = thread_rng().gen();
        println!("{}", hex::encode(rand));
    }
}
