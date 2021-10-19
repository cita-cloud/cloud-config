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

// use status_code::StatusCode;

// use crate::status_code::StatusCode;

pub trait Kms {
    fn create_kms_db(db_path: String, password: String) -> Self;
    fn generate_key_pair(&self, description: String) -> (u64, Vec<u8>);
}
