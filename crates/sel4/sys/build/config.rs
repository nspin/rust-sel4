//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

pub(crate) use sel4_config_data::get_config;

pub(crate) fn get_cfg_bool(key: &str) -> bool {
    get_config().get(key).unwrap().as_bool().unwrap()
}

pub(crate) fn get_cfg_usize(key: &str) -> usize {
    get_config()
        .get(key)
        .unwrap()
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}

pub(crate) fn get_word_size() -> usize {
    get_cfg_usize("WORD_SIZE")
}
