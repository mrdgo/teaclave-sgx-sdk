// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

// The Intel SGX SDK version recorded in `g_global_data`. `sgx_sign` refuses to
// sign an enclave whose recorded SDK version differs from its own, so the
// build script detects the installed SDK and generates `MAJOR_VERSION`,
// `MINOR_VERSION` and `REVISION_VERSION` (see `build.rs`).
include!(concat!(env!("OUT_DIR"), "/sdk_version.rs"));

pub const VERSION_UINT: usize = (MAJOR_VERSION << 32) | (MINOR_VERSION << 16) | REVISION_VERSION;
