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
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, CertificateSigningRequest, DistinguishedName,
    DnType, DnValue, IsCa, KeyPair, PKCS_ECDSA_P256_SHA256,
};
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io, path};
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
        .unwrap_or_else(|_| panic!("open file({:?}) failed.", path.as_ref().to_str()));
    file.write_all(toml::to_string_pretty(&toml).unwrap().as_bytes())
        .unwrap();
    file.write_all(b"\n").unwrap();
}

pub fn read_chain_config(path: impl AsRef<path::Path>) -> Result<ChainConfig, Error> {
    let buffer = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("Error while loading config: [{err}]"));
    toml::from_str::<ChainConfig>(&buffer)
}

pub fn read_node_config(path: impl AsRef<path::Path>) -> Result<NodeConfig, Error> {
    let buffer = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("Error while loading config: [{err}]"));
    toml::from_str::<NodeConfig>(&buffer)
}

pub fn write_toml<T: serde::Serialize>(content: T, path: impl AsRef<path::Path>) {
    let toml = Value::try_from(content).unwrap();

    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path.as_ref())
        .unwrap_or_else(|_| panic!("open file({:?}) failed.", path.as_ref().to_str()));
    file.write_all(toml::to_string_pretty(&toml).unwrap().as_bytes())
        .unwrap();
    file.write_all(b"\n").unwrap();
}

pub fn write_file(content: &[u8], path: impl AsRef<path::Path>) {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path.as_ref())
        .unwrap_or_else(|_| panic!("open file({:?}) failed.", path.as_ref().to_str()));
    file.write_all(content).unwrap();
}

pub fn touch_file(path: impl AsRef<path::Path>) {
    fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path.as_ref())
        .unwrap_or_else(|_| panic!("touch file({:?}) failed.", path.as_ref().to_str()));
}

pub fn read_file(path: impl AsRef<path::Path>) -> std::io::Result<String> {
    let mut f = fs::File::open(path)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}

pub fn unix_now() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    since_the_epoch.as_millis() as u64
}

const HASH_BYTES_LEN: usize = 32;

pub fn sm3_hash(input: &[u8]) -> [u8; HASH_BYTES_LEN] {
    libsm::sm3::hash::Sm3Hash::new(input).get_hash()
}

pub fn ca_cert() -> (Certificate, String, String) {
    let mut params = CertificateParams::default();
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);

    let keypair = KeyPair::generate(&PKCS_ECDSA_P256_SHA256).unwrap();
    params.key_pair.replace(keypair);

    let mut dn = DistinguishedName::new();
    dn.push(DnType::OrganizationName, "CITAHub");
    dn.push(
        DnType::CommonName,
        DnValue::PrintableString("CA".to_string()),
    );
    params.distinguished_name = dn;

    let cert = Certificate::from_params(params).unwrap();
    let cert_pem = cert.serialize_pem_with_signer(&cert).unwrap();
    let key_pem = cert.serialize_private_key_pem();
    (cert, cert_pem, key_pem)
}

pub fn restore_ca_cert(ca_cert_pem: &str, ca_key_pem: &str) -> Certificate {
    let ca_key_pair = KeyPair::from_pem(ca_key_pem).unwrap();
    let ca_param = CertificateParams::from_ca_cert_pem(ca_cert_pem, ca_key_pair).unwrap();

    Certificate::from_params(ca_param).unwrap()
}

pub fn create_csr(domain: &str) -> (String, String) {
    let subject_alt_names = vec![domain.into()];
    let mut params = CertificateParams::new(subject_alt_names);

    let mut dn = DistinguishedName::new();
    dn.push(DnType::OrganizationName, "CITAHub");
    dn.push(DnType::CommonName, DnValue::PrintableString(domain.into()));
    params.distinguished_name = dn;

    let keypair = KeyPair::generate(&PKCS_ECDSA_P256_SHA256).unwrap();
    params.key_pair.replace(keypair);

    let cert = Certificate::from_params(params).unwrap();

    let csr_pem = cert.serialize_request_pem().unwrap();
    let key_pem = cert.serialize_private_key_pem();

    (csr_pem, key_pem)
}

pub fn sign_csr(csr_pem: &str, ca_cert: &Certificate) -> String {
    let csr = CertificateSigningRequest::from_pem(csr_pem).unwrap();
    csr.serialize_pem_with_signer(ca_cert).unwrap()
}

pub fn find_micro_service(chain_config: &ChainConfig, service_name: &str) -> bool {
    for micro_service in &chain_config.micro_service_list {
        if micro_service.image == service_name {
            return true;
        }
    }
    false
}

pub fn remove_0x(s: &str) -> &str {
    s.strip_prefix("0x").unwrap_or(s)
}

pub fn check_address(s: &str) -> &str {
    let addr = s.strip_prefix("0x").unwrap_or(s);
    if addr.len() != 40 && addr.len() != 96 {
        panic!("wrong address, please check!")
    };
    addr
}

pub fn copy_dir_all(src: impl AsRef<path::Path>, dst: impl AsRef<path::Path>) -> io::Result<()> {
    let _ = fs::create_dir_all(&dst);
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();

        if let Ok(file_name) = entry.file_name().into_string() {
            if file_name.starts_with('.') {
                continue;
            }
        } else {
            continue;
        }

        if path.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn svc_name(chain_name: &str, domain: &str) -> String {
    format!("{chain_name}-{domain}")
}

pub fn clap_about() -> String {
    let name = env!("CARGO_PKG_NAME").to_string();
    let version = env!("CARGO_PKG_VERSION");
    let authors = env!("CARGO_PKG_AUTHORS");
    name + " " + version + "\n" + authors
}
