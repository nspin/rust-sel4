//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs;

use anyhow::Result;
use rkyv::rancor;
use rkyv::util::AlignedVec;

use sel4_capdl_initializer_types::InputSpec;

mod args;
mod render_elf;
mod reserialize_spec;

use args::Args;

// HACK hardcoded
const GRANULE_SIZE_BITS: u8 = 12;

type ArchiveAlignedVec = AlignedVec<16>;

fn main() -> Result<()> {
    let args = Args::parse()?;

    if args.verbose {
        eprintln!("{args:#?}");
    }

    let initializer_elf_buf = fs::read(&args.initializer_elf_path)?;
    let spec_json = fs::read_to_string(&args.spec_json_path)?;
    let fill_dir_path = &args.fill_dir_path;
    let out_file_path = &args.out_file_path;
    let object_names_level = &args.object_names_level;
    let embed_frames = args.embed_frames;

    let input_spec = InputSpec::parse(&spec_json);

    let (output_spec, embedded_frame_data) = reserialize_spec::reserialize_spec(
        &input_spec,
        fill_dir_path,
        object_names_level,
        embed_frames,
        GRANULE_SIZE_BITS,
        args.verbose,
    );

    let spec_data: ArchiveAlignedVec = rkyv::to_bytes::<rancor::Error>(&output_spec).unwrap();

    let render_elf_args = render_elf::RenderElfArgs {
        spec_data: &spec_data,
        spec_data_alignment: ArchiveAlignedVec::ALIGNMENT,
        embedded_frame_data: &embedded_frame_data,
        embedded_frame_data_alignment: GRANULE_SIZE_BITS.into(),
    };

    let rendered_initializer_elf_buf = match object::File::parse(&*initializer_elf_buf).unwrap() {
        object::File::Elf32(initializer_elf) => render_elf_args.call_with(&initializer_elf),
        object::File::Elf64(initializer_elf) => render_elf_args.call_with(&initializer_elf),
        _ => {
            panic!()
        }
    };

    fs::write(out_file_path, rendered_initializer_elf_buf)?;
    Ok(())
}
