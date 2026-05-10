//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs::{self, File};

use anyhow::Result;
use num::NumCast;
use num::{Integer, PrimInt, traits::WrappingSub};
use object::ReadRef;
use object::read::elf::{ElfFile, ProgramHeader};
use object::{
    Endianness,
    elf::{FileHeader32, FileHeader64},
    read::elf::FileHeader,
};
use rkyv::util::AlignedVec;
use serde::Serialize;

use sel4_config_types::Configuration;
use sel4_patch_elf::{FileHeaderExt, Patching};
use sel4_phdrs_constants::PT_SEL4_KERNEL_LOADER_PAYLOAD;

mod args;
mod page_tables;
mod platform_info;
mod serialize_payload;
mod utils;

use args::Args;
use platform_info::PlatformInfoForBuildSystem;

use crate::utils::{virt_footprint, with_elf};

type ArchiveAlignedVec = AlignedVec;

fn main() -> Result<()> {
    let args = Args::parse()?;

    if args.verbose {
        eprintln!("{args:#?}");
    }

    let sel4_config: Configuration =
        serde_json::from_reader(File::open(&args.sel4_config_path).unwrap()).unwrap();

    let word_size = sel4_config
        .get("WORD_SIZE")
        .unwrap()
        .as_str()
        .unwrap()
        .parse::<usize>()
        .unwrap();

    match word_size {
        32 => continue_with_word_size::<FileHeader32<Endianness>>(&args),
        64 => continue_with_word_size::<FileHeader64<Endianness>>(&args),
        _ => {
            panic!()
        }
    }
}

fn continue_with_word_size<T>(args: &Args) -> Result<()>
where
    T: FileHeader<Word: PrimInt + WrappingSub + Integer + Serialize, Endian = Endianness>
        + FileHeaderExt,
{
    let platform_info: PlatformInfoForBuildSystem =
        serde_yaml::from_reader(fs::File::open(&args.platform_info_path).unwrap()).unwrap();

    let loader_bytes = fs::read(&args.loader_path)?;

    let payload = serialize_payload::serialize_payload::<T>(
        &args.kernel_path,
        &args.app_path,
        &args.dtb_path,
        &platform_info,
    );

    let payload_data: ArchiveAlignedVec = payload.to_bytes().unwrap();

    let final_loader = {
        let orig_elf = ElfFile::<T>::parse(&loader_bytes).unwrap();
        let mut patching = Patching::new(&orig_elf);
        {
            let mut addr_slot = None;
            patching.add_data_segment(page_tables::ALIGN, |vaddr| {
                addr_slot = Some(vaddr);
                page_tables::mk_loader_map(vaddr, &platform_info)
            });
            let addr = <T::Word as NumCast>::from(addr_slot.unwrap()).unwrap();
            patching.patch_word("loader_level_0_table", addr); // addr.unwrap()
        }
        {
            let mut addr_slot = None;
            patching.add_data_segment(page_tables::ALIGN, |vaddr| {
                addr_slot = Some(vaddr);
                with_elf::<T, _, _>(&args.kernel_path, |elf| {
                    let phys_to_virt_offset = kernel_phys_to_virt_offset(elf);
                    let virt_range = virt_footprint(elf);
                    let phys_range = virt_range.start.wrapping_add(virt_range.start)
                        ..virt_range.end.wrapping_add(virt_range.end);
                    page_tables::mk_kernel_map(vaddr, phys_range, phys_to_virt_offset)
                })
            });
            let addr = <T::Word as NumCast>::from(addr_slot.unwrap()).unwrap();
            patching.patch_word("kernel_boot_level_0_table", addr); // addr.unwrap()
        }
        patching.add_data_segment_with_meta_phdr(
            PT_SEL4_KERNEL_LOADER_PAYLOAD,
            ArchiveAlignedVec::ALIGNMENT.try_into().unwrap(),
            &payload_data,
        );
        patching.finalize()
    };

    let out_file_path = &args.out_file_path;

    fs::write(out_file_path, final_loader)?;
    Ok(())
}

fn kernel_phys_to_virt_offset<'a, T: FileHeader, R: ReadRef<'a>>(elf: &ElfFile<'a, T, R>) -> u64 {
    let endian = elf.endian();
    let phdr = utils::loadable_segments(elf)
        .next()
        .unwrap()
        .elf_program_header();
    let vaddr = phdr.p_vaddr(endian).into();
    let paddr = phdr.p_paddr(endian).into();
    vaddr.wrapping_sub(paddr)
}
