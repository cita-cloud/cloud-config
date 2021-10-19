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

use clap::{AppSettings, Parser};

use crate::append::{AppendOpts, execute_append};
use crate::create::{CreateOpts, execute_create};
use crate::delete::{DeleteOpts, execute_delete};

mod append;
mod config;
mod create;
mod delete;
mod traits;
mod util;
mod error;
mod status_code;

#[derive(Parser)]
#[clap(version = "6.3.0", author = "Yieazy")]
struct Opts {
    #[clap(subcommand)]
    sub_cmd: SubCommand,
}

#[derive(Parser)]
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
}

fn main() {
    ::std::env::set_var("RUST_BACKTRACE", "full");

    let opts: Opts = Opts::parse();

    match opts.sub_cmd {
        SubCommand::Create(opts) => execute_create(opts).unwrap(),
        SubCommand::Append(opts) => execute_append(opts).unwrap(),
        SubCommand::Delete(opts) => execute_delete(opts).unwrap(),
    }
}
