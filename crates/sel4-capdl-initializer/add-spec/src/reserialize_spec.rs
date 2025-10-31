//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::ops::Range;
use std::path::Path;

use sel4_capdl_initializer_types::*;

pub fn reserialize_spec(
    input_spec: &InputSpec,
    fill_dir_path: impl AsRef<Path>,
    object_names_level: &ObjectNamesLevel,
    embed_frames: bool,
    granule_size_bits: u8,
    verbose: bool,
) -> (SpecForInitializer, Vec<u8>) {
    let granule_size = 1 << granule_size_bits;

    let fill_map = input_spec.collect_fill(&[fill_dir_path]);

    let mut sources = SourcesBuilder::new();
    let mut num_embedded_frames = 0;
    let output_spec: SpecForInitializer = input_spec
        .traverse_names_with_context(|named_obj| object_names_level.apply(named_obj).cloned())
        .split_embedded_frames(embed_frames, granule_size_bits)
        .traverse_data(|key| DeflatedBytesContent::pack(fill_map.get(key)))
        .traverse_embedded_frames(|fill| {
            num_embedded_frames += 1;
            sources.align_to(granule_size);
            let range = sources.append(&fill_map.get_frame(granule_size, fill));
            EmbeddedFrameOffset::new(u64::from_usize(range.start))
        });

    if verbose {
        eprintln!("embedded frames count: {num_embedded_frames}");
    }

    (output_spec, sources.build())
}

struct SourcesBuilder {
    buf: Vec<u8>,
}

impl SourcesBuilder {
    fn new() -> Self {
        Self { buf: vec![] }
    }

    fn build(self) -> Vec<u8> {
        self.buf
    }

    fn align_to(&mut self, align: usize) {
        assert!(align.is_power_of_two());
        self.buf.resize(self.buf.len().next_multiple_of(align), 0);
    }

    fn append(&mut self, bytes: &[u8]) -> Range<usize> {
        let start = self.buf.len();
        self.buf.extend(bytes);
        let end = self.buf.len();
        start..end
    }
}
