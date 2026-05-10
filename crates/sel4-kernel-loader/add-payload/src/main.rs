//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs::{self, File};

use anyhow::Result;
use num::traits::NumCast;
use object::elf::{FileHeader32, FileHeader64};
use object::read::elf::{ElfFile, FileHeader, ProgramHeader};
use object::{Endianness, ReadRef};
use rkyv::util::AlignedVec;

use sel4_config_types::Configuration;
use sel4_patch_elf::{FileHeaderExt, Patching};
use sel4_phdrs_constants::PT_SEL4_KERNEL_LOADER_PAYLOAD;

mod args;
mod maps;
mod page_tables;
mod platform_info;
mod serialize_payload;
mod utils;

use args::Args;
use platform_info::PlatformInfoForBuildSystem;

use crate::maps::SchemeExt;
use crate::page_tables::schemes;
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
        .get("SEL4_ARCH")
        .unwrap()
        .as_str()
        .unwrap();

    match word_size {
        "aarch32" => continue_with_config::<FileHeader32<Endianness>, schemes::AArch32>(&args),
        "aarch64" => continue_with_config::<FileHeader64<Endianness>, schemes::AArch64>(&args),
        "riscv32" => continue_with_config::<FileHeader32<Endianness>, schemes::RiscV32Sv32>(&args),
        "riscv64" => continue_with_config::<FileHeader64<Endianness>, schemes::RiscV64Sv39>(&args),
        _ => {
            panic!()
        }
    }
}

fn continue_with_config<T, S>(args: &Args) -> Result<()>
where
    T: FileHeaderExt<Word: NumCast>,
    S: SchemeExt + 'static,
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
            patching.add_data_segment(maps::ALIGN, |vaddr| {
                addr_slot = Some(vaddr);
                maps::mk_loader_map::<S>(vaddr, &platform_info)
            });
            let addr = <T::Word as NumCast>::from(addr_slot.unwrap()).unwrap();
            patching.patch_word("x_loader_level_0_table", addr);
        }
        {
            let mut addr_slot = None;
            patching.add_data_segment(maps::ALIGN, |vaddr| {
                with_elf::<T, _, _>(&args.kernel_path, |elf| {
                    let phys_to_virt_offset = kernel_phys_to_virt_offset(elf);
                    let virt_range = virt_footprint(elf);
                    let (bytes, root_vaddr) =
                        maps::mk_kernel_map::<S>(vaddr, virt_range, phys_to_virt_offset);
                    addr_slot = Some(root_vaddr);
                    bytes
                })
            });
            let addr = <T::Word as NumCast>::from(addr_slot.unwrap()).unwrap();
            patching.patch_word("kernel_boot_level_0_table", addr);
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
