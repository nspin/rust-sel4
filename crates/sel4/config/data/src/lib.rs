//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(unused_imports)]

use std::sync::LazyLock;

pub use sel4_config_types::Config;

pub fn get_config() -> &'static Config {
    &CONFIG
}

static CONFIG: LazyLock<Config> = LazyLock::new(|| serde_json::from_str(CONFIG_JSON).unwrap());

const CONFIG_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/config.json"));
