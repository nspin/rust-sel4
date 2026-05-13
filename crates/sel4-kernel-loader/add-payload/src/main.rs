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
mod page_tables2;
mod platform_info;
mod serialize_payload;
mod utils;

use args::Args;
use platform_info::PlatformInfoForBuildSystem;

use crate::page_tables2::Scheme;
use crate::utils::{virt_footprint, with_elf};

type ArchiveAlignedVec = AlignedVec;

fn main() -> Result<()> {
    let args = Args::parse()?;

    if args.verbose {
        eprintln!("{args:#?}");
    }

    let sel4_config: Configuration =
        serde_json::from_reader(File::open(&args.sel4_config_path).unwrap()).unwrap();

    let word_size = sel4_config.get("WORD_SIZE").unwrap().as_str().unwrap();

    match word_size {
        "32" => continue_with_config::<FileHeader32<Endianness>>(&args, &sel4_config),
        "64" => continue_with_config::<FileHeader64<Endianness>>(&args, &sel4_config),
        _ => {
            panic!()
        }
    }
}

fn continue_with_config<T>(args: &Args, sel4_config: &Configuration) -> Result<()>
where
    T: FileHeaderExt<Word: NumCast>,
{
    let platform_info: PlatformInfoForBuildSystem =
        serde_yaml::from_reader(fs::File::open(&args.platform_info_path).unwrap()).unwrap();

    let loader_bytes = fs::read(&args.loader_path)?;

    let scheme = Scheme::from_config(sel4_config);

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
        if sel4_config.get("ARCH_ARM").unwrap().as_bool().unwrap() {
            let mut addr_slot = None;
            patching.add_data_segment(maps::ALIGN, |vaddr| {
                let (bytes, root_vaddr) = maps::mk_loader_map(&scheme, vaddr, &platform_info);
                addr_slot = Some(root_vaddr);
                bytes
            });
            let addr = <T::Word as NumCast>::from(addr_slot.unwrap()).unwrap();
            patching.patch_word("loader_level_0_table", addr);
        }
        {
            let mut addr_slot = None;
            patching.add_data_segment(maps::ALIGN, |vaddr| {
                with_elf::<T, _, _>(&args.kernel_path, |elf| {
                    let phys_to_virt_offset = kernel_phys_to_virt_offset(elf, scheme.vaddr_mask());
                    let virt_range = virt_footprint(elf);
                    let masked_virt_addr_range = virt_range.start & scheme.vaddr_mask()
                        ..virt_range.end & scheme.vaddr_mask();
                    let (bytes, root_vaddr) = maps::mk_kernel_map(
                        &scheme,
                        vaddr,
                        masked_virt_addr_range,
                        phys_to_virt_offset,
                    );
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

fn kernel_phys_to_virt_offset<'a, T: FileHeader, R: ReadRef<'a>>(
    elf: &ElfFile<'a, T, R>,
    vaddr_mask: u64,
) -> u64 {
    let endian = elf.endian();
    let phdr = utils::loadable_segments(elf)
        .next()
        .unwrap()
        .elf_program_header();
    let vaddr = phdr.p_vaddr(endian).into() & vaddr_mask;
    let paddr = phdr.p_paddr(endian).into();
    vaddr.wrapping_sub(paddr)
}

// fn elf_phys_addr_range<'a, T: FileHeader<Word: Ord + num::CheckedAdd>, R: ReadRef<'a>>(elf: &ElfFile<'a, T, R>) -> std::ops::Range<u64> {
//     let endian = elf.endian();
//     let virt_min = elf
//         .elf_program_headers()
//         .iter()
//         .filter(|phdr| phdr.p_type(endian) == object::elf::PT_LOAD)
//         .map(|phdr| phdr.p_paddr(endian))
//         .min()
//         .unwrap();
//     let virt_max = elf
//         .elf_program_headers()
//         .iter()
//         .filter(|phdr| phdr.p_type(endian) == object::elf::PT_LOAD)
//         .map(|phdr| {
//             num::CheckedAdd::checked_add(&phdr.p_paddr(endian), &phdr.p_memsz(endian))
//                 .unwrap()
//         })
//         .max()
//         .unwrap();
//     virt_min.into()..virt_max.into()
// }

// fn elf_phys_to_vaddr_offset<'a, T: FileHeader, R: ReadRef<'a>>(elf: &ElfFile<'a, FileHeaTder, R>) -> u64 {
//     let endian = elf.endian();
//     unified(
//         elf.elf_program_headers()
//             .iter()
//             .filter(|phdr| phdr.p_type(endian) == object::elf::PT_LOAD)
//             .map(|phdr| {
//                 let paddr = phdr.p_paddr(endian).into();
//                 let vaddr_with_extension: u64 = phdr.p_vaddr(endian).into();
//                 let vaddr = vaddr_with_extension & SchemeHelpers::<S>::vaddr_mask();
//                 phys_to_virt_offset_for(paddr, vaddr)
//             }),
//     )
// }

// fn unified<T: Eq>(mut it: impl Iterator<Item = T>) -> T {
//     let first = it.next().unwrap();
//     assert!(it.all(|subsequent| subsequent == first));
//     first
// }

// fn phys_to_virt_offset_for(paddr: u64, vaddr: u64) -> u64 {
//     vaddr.wrapping_sub(paddr)
// }

// fn virt_to_phys(vaddr: u64, phys_to_virt_offset: u64) -> u64 {
//     vaddr.wrapping_sub(phys_to_virt_offset)
// }
