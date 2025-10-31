//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::path::Path;

use sel4_capdl_initializer_types::*;

pub fn reserialize_spec(
    input_spec: &InputSpec,
    fill_dirs: &[impl AsRef<Path>],
    object_names_level: &ObjectNamesLevel,
    embed_frames: bool,
    granule_size_bits: u8,
) -> (SpecForInitializer, EmbeddedFramesData) {
    let granule_size = 1 << granule_size_bits;

    let fill_map = input_spec.collect_fill(fill_dirs);

    let mut embedded_frames_data = EmbeddedFramesData::new(granule_size);

    let output_spec: SpecForInitializer = input_spec
        .traverse_names_with_context(|named_obj| object_names_level.apply(named_obj).cloned())
        .split_embedded_frames(embed_frames, granule_size_bits)
        .traverse_data(|key| DeflatedBytesContent::pack(fill_map.get(key)))
        .traverse_embedded_frames(|fill| {
            embedded_frames_data.align_to(granule_size);
            let start = embedded_frames_data.append(&fill_map.get_frame(granule_size, fill));
            EmbeddedFrameOffset::new(start.try_into().unwrap())
        });

    (output_spec, embedded_frames_data)
}

pub(crate) struct EmbeddedFramesData {
    pub(crate) alignment: usize,
    pub(crate) data: Vec<u8>,
}

impl EmbeddedFramesData {
    fn new(alignment: usize) -> Self {
        Self {
            alignment,
            data: vec![],
        }
    }

    fn align_to(&mut self, align: usize) {
        assert!(align.is_power_of_two());
        self.data.resize(self.data.len().next_multiple_of(align), 0);
    }

    fn append(&mut self, bytes: &[u8]) -> usize {
        let start = self.data.len();
        self.data.extend(bytes);
        start
    }
}
