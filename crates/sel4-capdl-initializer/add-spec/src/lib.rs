//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::path::Path;

use rkyv::rancor;
use rkyv::util::AlignedVec;

use sel4_capdl_initializer_types::InputSpec;

mod render_elf;
mod reserialize_spec;

// HACK hardcoded
const GRANULE_SIZE_BITS: u8 = 12;

type ArchiveAlignedVec = AlignedVec<16>;

pub fn add_spec(
    initializer_without_spec: &[u8],
    spec: &InputSpec,
    fill_dirs: &[impl AsRef<Path>],
    object_names_level: &ObjectNamesLevel,
    embed_frames: bool,
) -> Vec<u8> {
    let (output_spec, embedded_frame_data) = reserialize_spec::reserialize_spec(
        spec,
        fill_dirs,
        object_names_level,
        embed_frames,
        GRANULE_SIZE_BITS,
    );

    let output_spec_data: ArchiveAlignedVec =
        rkyv::to_bytes::<rancor::Error>(&output_spec).unwrap();

    let render_elf_args = render_elf::RenderElfArgs {
        spec_data: &output_spec_data,
        spec_data_alignment: 1 << ArchiveAlignedVec::ALIGNMENT,
        embedded_frame_data: &embedded_frame_data.data,
        embedded_frame_data_alignment: embedded_frame_data.alignment,
    };

    match object::File::parse(initializer_without_spec).unwrap() {
        object::File::Elf32(initializer_elf) => render_elf_args.call_with(&initializer_elf),
        object::File::Elf64(initializer_elf) => render_elf_args.call_with(&initializer_elf),
        _ => {
            panic!()
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ObjectNamesLevel {
    All,
    JustTcbs,
    None,
}
