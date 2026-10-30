//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use object::{CompressionFormat, Object, ObjectSection};

use sel4_phdrs::{PT_SEL4_EMBEDDED_DEBUG_INFO, locate_phdrs};

use sel4_phdrs_patched as _;

pub type Context = addr2line::Context<gimli::EndianSlice<'static, gimli::RunTimeEndian>>;

#[derive(thiserror::Error, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    #[error("ELF error")]
    ObjectError(#[from] object::Error),
    #[error("DWARF error")]
    GimliError(#[from] gimli::Error),
    #[error("compressed section: {0}")]
    CompressedSection(&'static str),
}

pub fn get_context() -> Result<Context, Error> {
    let embedded_debug_info = unsafe {
        locate_phdrs()
            .unwrap()
            .find_by_type(PT_SEL4_EMBEDDED_DEBUG_INFO)
            .unwrap()
            .bytes()
    };
    let obj = object::File::parse(embedded_debug_info)?;
    let endian = match obj.endianness() {
        object::Endianness::Little => gimli::RunTimeEndian::Little,
        object::Endianness::Big => gimli::RunTimeEndian::Big,
    };
    let dwarf = gimli::Dwarf::load(|id| {
        let data = match obj.section_by_name(id.name()) {
            Some(section) => {
                let compressed = section.compressed_data()?;
                if compressed.format != CompressionFormat::None {
                    return Err(Error::CompressedSection(id.name()));
                }
                compressed.data
            }
            None => &[],
        };
        Ok(gimli::EndianSlice::new(data, endian))
    })?;
    Ok(Context::from_dwarf(dwarf)?)
}
