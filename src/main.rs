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

use clap::Parser;

use crate::append_node::{execute_append_node, AppendNodeOpts};
use crate::append_validator::{execute_append_validator, AppendValidatorOpts};
use crate::create_ca::{execute_create_ca, CreateCAOpts};
use crate::create_csr::{execute_create_csr, CreateCSROpts};
use crate::delete_chain::{execute_delete_chain, DeleteChainOpts};
use crate::delete_node::{execute_delete_node, DeleteNodeOpts};
use crate::env_dev::{
    execute_append_dev, execute_create_dev, execute_delete_dev, AppendDevOpts, CreateDevOpts,
    DeleteDevOpts,
};
use crate::env_k8s::{
    execute_append_k8s, execute_create_k8s, execute_delete_k8s, AppendK8sOpts, CreateK8sOpts,
    DeleteK8sOpts,
};
use crate::import_account::{execute_import_account, ImportAccountOpts};
use crate::init_chain::{execute_init_chain, InitChainOpts};
use crate::init_chain_config::{execute_init_chain_config, InitChainConfigOpts};
use crate::init_node::{execute_init_node, InitNodeOpts};
use crate::migrate::{execute_migrate, MigrateOpts};
use crate::new_account::{execute_new_account, NewAccountOpts};
use crate::set_admin::{execute_set_admin, SetAdminOpts};
use crate::set_nodelist::{execute_set_nodelist, SetNodeListOpts};
use crate::set_validators::{execute_set_validators, SetValidatorsOpts};
use crate::sign_csr::{execute_sign_csr, SignCSROpts};
use crate::update_node::{execute_update_node, UpdateNodeOpts};

mod append_node;
mod append_validator;
mod config;
mod constant;
mod create_ca;
mod create_csr;
mod delete_chain;
mod delete_node;
mod env_dev;
mod env_k8s;
mod error;
mod import_account;
mod init_chain;
mod init_chain_config;
mod init_node;
mod migrate;
mod new_account;
mod set_admin;
mod set_nodelist;
mod set_validators;
mod sign_csr;
mod traits;
mod update_node;
mod util;

#[derive(Parser)]
#[clap(version = "6.3.0", author = "Rivtower Technologies.")]
struct Opts {
    #[clap(subcommand)]
    sub_cmd: SubCommand,
}

#[allow(clippy::large_enum_variant)]
#[derive(Parser)]
enum SubCommand {
    /// init a chain
    #[clap(name = "init-chain")]
    InitChain(InitChainOpts),
    /// init chain config
    #[clap(name = "init-chain-config")]
    InitChainConfig(InitChainConfigOpts),
    /// set admin of chain
    #[clap(name = "set-admin")]
    SetAdmin(SetAdminOpts),
    /// set validators of chain
    #[clap(name = "set-validators")]
    SetValidators(SetValidatorsOpts),
    /// append a validator into chain
    #[clap(name = "append-validator")]
    AppendValidator(AppendValidatorOpts),
    /// set node list
    #[clap(name = "set-nodelist")]
    SetNodeList(SetNodeListOpts),
    /// append a node into chain
    #[clap(name = "append-node")]
    AppendNode(AppendNodeOpts),
    /// delete a node from chain
    #[clap(name = "delete-node")]
    DeleteNode(DeleteNodeOpts),
    /// init node
    #[clap(name = "init-node")]
    InitNode(InitNodeOpts),
    /// update node
    #[clap(name = "update-node")]
    UpdateNode(UpdateNodeOpts),
    /// delete a chain
    #[clap(name = "delete-chain")]
    DeleteChain(DeleteChainOpts),
    /// new account
    #[clap(name = "new-account")]
    NewAccount(NewAccountOpts),
    /// import account
    #[clap(name = "import-account")]
    ImportAccount(ImportAccountOpts),
    /// create CA
    #[clap(name = "create-ca")]
    CreateCA(CreateCAOpts),
    /// create csr
    #[clap(name = "create-csr")]
    CreateCSR(CreateCSROpts),
    /// sign csr
    #[clap(name = "sign-csr")]
    SignCSR(SignCSROpts),
    /// create config in env dev
    #[clap(name = "create-dev")]
    CreateDev(CreateDevOpts),
    /// append node in env dev
    #[clap(name = "append-dev")]
    AppendDev(AppendDevOpts),
    /// delete node in env dev
    #[clap(name = "delete-dev")]
    DeleteDev(DeleteDevOpts),
    /// create config in env k8s
    #[clap(name = "create-k8s")]
    CreateK8s(CreateK8sOpts),
    /// append node in env k8s
    #[clap(name = "append-k8s")]
    AppendK8s(AppendK8sOpts),
    /// delete node in env k8s
    #[clap(name = "delete-k8s")]
    DeleteK8s(DeleteK8sOpts),
    /// migrate CITA-Cloud chain from 6.1.0 to 6.3.0
    #[clap(name = "migrate")]
    Migrate(MigrateOpts),
}

fn main() {
    ::std::env::set_var("RUST_BACKTRACE", "full");

    let opts: Opts = Opts::parse();

    match opts.sub_cmd {
        SubCommand::InitChain(opts) => execute_init_chain(opts).unwrap(),
        SubCommand::InitChainConfig(opts) => execute_init_chain_config(opts).unwrap(),
        SubCommand::SetAdmin(opts) => execute_set_admin(opts).unwrap(),
        SubCommand::SetValidators(opts) => execute_set_validators(opts).unwrap(),
        SubCommand::AppendValidator(opts) => execute_append_validator(opts).unwrap(),
        SubCommand::SetNodeList(opts) => execute_set_nodelist(opts).unwrap(),
        SubCommand::AppendNode(opts) => execute_append_node(opts).unwrap(),
        SubCommand::DeleteNode(opts) => execute_delete_node(opts).unwrap(),
        SubCommand::InitNode(opts) => execute_init_node(opts).unwrap(),
        SubCommand::UpdateNode(opts) => execute_update_node(opts).unwrap(),
        SubCommand::DeleteChain(opts) => execute_delete_chain(opts).unwrap(),
        SubCommand::NewAccount(opts) => execute_new_account(opts).map(|_| ()).unwrap(),
        SubCommand::ImportAccount(opts) => execute_import_account(opts).map(|_| ()).unwrap(),
        SubCommand::CreateCA(opts) => execute_create_ca(opts).map(|_| ()).unwrap(),
        SubCommand::CreateCSR(opts) => execute_create_csr(opts).map(|_| ()).unwrap(),
        SubCommand::SignCSR(opts) => execute_sign_csr(opts).map(|_| ()).unwrap(),
        SubCommand::CreateDev(opts) => execute_create_dev(opts).unwrap(),
        SubCommand::AppendDev(opts) => execute_append_dev(opts).unwrap(),
        SubCommand::DeleteDev(opts) => execute_delete_dev(opts).unwrap(),
        SubCommand::CreateK8s(opts) => execute_create_k8s(opts).unwrap(),
        SubCommand::AppendK8s(opts) => execute_append_k8s(opts).unwrap(),
        SubCommand::DeleteK8s(opts) => execute_delete_k8s(opts).unwrap(),
        SubCommand::Migrate(opts) => execute_migrate(opts).unwrap(),
    }
}
