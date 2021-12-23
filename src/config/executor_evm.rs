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

use crate::constant::{EXECUTOR, EXECUTOR_EVM};
use crate::traits::{TomlWriter, YmlWriter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ExecutorEvmConfig {
    pub executor_port: u16,
}

impl ExecutorEvmConfig {
    pub fn new(executor_port: u16) -> Self {
        Self { executor_port }
    }
}

impl TomlWriter for ExecutorEvmConfig {
    fn section(&self) -> String {
        EXECUTOR_EVM.to_string()
    }
}

impl YmlWriter for ExecutorEvmConfig {
    fn service(&self) -> String {
        EXECUTOR.to_string()
    }
}

#[cfg(test)]
mod executor_test {
    use super::*;

    #[test]
    fn basic_test() {
        let _ = std::fs::remove_file("example/config.toml");

        let config = ExecutorEvmConfig {
            executor_port: 51232,
        };

        config.write("example");
    }
}
