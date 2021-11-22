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

use clap::Clap;

use crate::append::{execute_append, AppendOpts};
use crate::create::{execute_create, CreateOpts};
use crate::create_ca::{execute_create_ca, CreateCAOpts};
use crate::create_cert::{execute_create_cert, CreateCertOpts};
use crate::delete::{execute_delete, DeleteOpts};
use crate::delete_chain::{execute_delete_chain, DeleteChainOpts};
use crate::init_chain::{execute_init_chain, InitChainOpts};
use crate::init_chain_config::{execute_init_chain_config, InitChainConfigOpts};
use crate::init_node::{execute_init_node, InitNodeOpts};
use crate::new_account::{execute_new_account, NewAccountOpts};
use crate::set_admin::{execute_set_admin, SetAdminOpts};
use crate::set_ca::{execute_set_ca, SetCAOpts};
use crate::set_nodelist::{execute_set_nodelist, SetNodeListOpts};
use crate::set_validators::{execute_set_validators, SetValidatorsOpts};
use crate::update_node::{execute_update_node, UpdateNodeOpts};

mod append;
mod config;
mod constant;
mod create;
mod create_ca;
mod create_cert;
mod delete;
mod delete_chain;
mod error;
mod init_chain;
mod init_chain_config;
mod init_node;
mod new_account;
mod set_admin;
mod set_ca;
mod set_nodelist;
mod set_validators;
mod traits;
mod update_node;
mod util;

#[derive(Clap)]
#[clap(version = "6.3.0", author = "Rivtower Technologies.")]
struct Opts {
    #[clap(subcommand)]
    sub_cmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    /// create config
    #[clap(name = "create")]
    Create(CreateOpts),
    /// append config
    #[clap(name = "append")]
    Append(AppendOpts),
    /// delete config
    #[clap(name = "delete")]
    Delete(DeleteOpts),
    /// init a chain
    #[clap(name = "init-chain")]
    InitChain(InitChainOpts),
    /// init chain config
    #[clap(name = "init-chain-config")]
    InitChainConfig(InitChainConfigOpts),
    /// set admin of chain
    #[clap(name = "set-admin")]
    SetAdmin(SetAdminOpts),
    /// set ca of chain
    #[clap(name = "set-ca")]
    SetCA(SetCAOpts),
    /// set validators of chain
    #[clap(name = "set-validators")]
    SetValidators(SetValidatorsOpts),
    /// set node list
    #[clap(name = "set-nodelist")]
    SetNodeList(SetNodeListOpts),
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
    /// create CA
    #[clap(name = "create-ca")]
    CreateCA(CreateCAOpts),
    /// create cert
    #[clap(name = "create-cert")]
    CreateCert(CreateCertOpts),
}

fn main() {
    ::std::env::set_var("RUST_BACKTRACE", "full");

    let opts: Opts = Opts::parse();

    match opts.sub_cmd {
        SubCommand::Create(opts) => execute_create(opts).unwrap(),
        SubCommand::Append(opts) => execute_append(opts).unwrap(),
        SubCommand::Delete(opts) => execute_delete(opts).unwrap(),
        SubCommand::InitChain(opts) => execute_init_chain(opts).unwrap(),
        SubCommand::InitChainConfig(opts) => execute_init_chain_config(opts).unwrap(),
        SubCommand::SetAdmin(opts) => execute_set_admin(opts).unwrap(),
        SubCommand::SetCA(opts) => execute_set_ca(opts).unwrap(),
        SubCommand::SetValidators(opts) => execute_set_validators(opts).unwrap(),
        SubCommand::SetNodeList(opts) => execute_set_nodelist(opts).unwrap(),
        SubCommand::InitNode(opts) => execute_init_node(opts).unwrap(),
        SubCommand::UpdateNode(opts) => execute_update_node(opts).unwrap(),
        SubCommand::DeleteChain(opts) => execute_delete_chain(opts).unwrap(),
        SubCommand::NewAccount(opts) => execute_new_account(opts).unwrap(),
        SubCommand::CreateCA(opts) => execute_create_ca(opts).unwrap(),
        SubCommand::CreateCert(opts) => execute_create_cert(opts).unwrap(),
    }
}
