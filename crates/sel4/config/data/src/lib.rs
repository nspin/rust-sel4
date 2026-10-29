//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(unused_imports)]

use std::sync::LazyLock;

use sel4_config_data_types::Config;

pub use sel4_config_data_types as types;

pub fn get_config() -> &'static Config {
    &CONFIG
}

static CONFIG: LazyLock<Config> = LazyLock::new(|| serde_json::from_str(CONFIG_JSON).unwrap());

const CONFIG_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/config.json"));

#[doc = concat!("```json\n", include_str!(concat!(env!("OUT_DIR"), "/config.json")), "\n```\n")]
pub mod rendered_json {}
