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

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Used when no Intel SGX SDK can be found at build time.
const DEFAULT_SDK_VERSION: (u32, u32, u32) = (2, 17, 101);

fn main() -> Result<(), &'static str> {
    println!("cargo:rerun-if-env-changed=SGX_MODE");
    println!("cargo:rerun-if-changed=build.rs");

    if cfg!(all(feature = "sim", feature = "hyper")) {
        println!("cargo:warning=Sim feature and hyper feature, only one or neither of them can be enabled.");
        return Err("Defined feature is illegal.");
    }

    let mode = env::var("SGX_MODE").unwrap_or_else(|_| "HW".to_string());
    match mode.as_ref() {
        "SIM" | "SW" => println!("cargo:rustc-cfg=feature=\"sim\""),
        "HYPER" => println!("cargo:rustc-cfg=feature=\"hyper\""),
        _ => (),
    }

    write_sdk_version()
}

/// Generates the SDK version constants used in `g_global_data`.
///
/// `sgx_sign` refuses to sign an enclave whose recorded SDK version differs
/// from its own, so the version has to match the SDK that will sign the
/// enclave. It is taken from, in order: the `SGX_SDK_VERSION` environment
/// variable (`MAJOR.MINOR.REVISION`), the `sgx_sign` tool of the SDK at
/// `SGX_SDK` (default `/opt/intel/sgxsdk`), or `DEFAULT_SDK_VERSION`.
fn write_sdk_version() -> Result<(), &'static str> {
    println!("cargo:rerun-if-env-changed=SGX_SDK_VERSION");
    println!("cargo:rerun-if-env-changed=SGX_SDK");

    let (major, minor, revision) = detect_sdk_version();
    let out_dir = env::var("OUT_DIR").map_err(|_| "OUT_DIR was not set")?;
    let contents = format!(
        "pub const MAJOR_VERSION: usize = {major};\n\
         pub const MINOR_VERSION: usize = {minor};\n\
         pub const REVISION_VERSION: usize = {revision};\n"
    );
    fs::write(PathBuf::from(out_dir).join("sdk_version.rs"), contents)
        .map_err(|_| "Failed to write sdk_version.rs")
}

fn detect_sdk_version() -> (u32, u32, u32) {
    if let Ok(version) = env::var("SGX_SDK_VERSION") {
        match parse_version(&version) {
            Some(parsed) => return parsed,
            None => println!(
                "cargo:warning=SGX_SDK_VERSION=\"{version}\" is not of the form MAJOR.MINOR.REVISION, ignoring it."
            ),
        }
    }

    let sdk = env::var("SGX_SDK").unwrap_or_else(|_| "/opt/intel/sgxsdk".to_string());
    let signer = PathBuf::from(sdk).join("bin/x64/sgx_sign");
    if let Ok(output) = Command::new(&signer).arg("-version").output() {
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        // "This is sgx_sign from Intel(R) Software Guard Extensions, version 2.30.100.1 for Linux."
        if let Some(parsed) = text.split("version ").nth(1).and_then(parse_version) {
            return parsed;
        }
        println!(
            "cargo:warning=Could not parse the SDK version from `{} -version`, using {}.{}.{}.",
            signer.display(),
            DEFAULT_SDK_VERSION.0,
            DEFAULT_SDK_VERSION.1,
            DEFAULT_SDK_VERSION.2
        );
    }
    DEFAULT_SDK_VERSION
}

/// Parses the leading `MAJOR.MINOR.REVISION` of a version string, ignoring
/// anything after the third component.
fn parse_version(text: &str) -> Option<(u32, u32, u32)> {
    let mut parts = text
        .trim()
        .split(|c: char| !c.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .map(|part| part.parse::<u32>().ok());
    Some((parts.next()??, parts.next()??, parts.next()??))
}
