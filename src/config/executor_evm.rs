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

use crate::constant::EXECUTOR_EVM;
use crate::traits::TomlWriter;
use serde::{Deserialize, Serialize};

use super::log_config::LogConfig;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ExecutorEvmConfig {
    pub domain: String,
    pub executor_port: u16,
    pub metrics_port: u16,
    pub enable_metrics: bool,
    pub log_config: LogConfig,
}

impl ExecutorEvmConfig {
    pub fn new(
        domain: String,
        executor_port: u16,
        metrics_port: u16,
        enable_metrics: bool,
        log_config: LogConfig,
    ) -> Self {
        Self {
            domain,
            executor_port,
            metrics_port,
            enable_metrics,
            log_config,
        }
    }
}

impl TomlWriter for ExecutorEvmConfig {
    fn section(&self) -> String {
        EXECUTOR_EVM.to_string()
    }
}

#[cfg(test)]
mod executor_test {
    use super::*;

    #[test]
    fn basic_test() {
        let _ = std::fs::remove_file("example/config.toml");

        let config = ExecutorEvmConfig {
            domain: "test-chain-0".to_string(),
            executor_port: 51232,
            metrics_port: 61232,
            enable_metrics: true,
            log_config: LogConfig::default(),
        };

        config.write("example");
    }
}
