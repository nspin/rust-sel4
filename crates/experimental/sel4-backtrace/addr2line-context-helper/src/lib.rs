//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use addr2line::Context as AbstractContext;
use object::{CompressionFormat, Object, ObjectSection};

pub type Context<'data> = AbstractContext<gimli::EndianSlice<'data, gimli::RunTimeEndian>>;

#[derive(thiserror::Error, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    #[error("ELF error")]
    ObjectError(#[from] object::Error),
    #[error("DWARF error")]
    GimliError(#[from] gimli::Error),
    #[error("compressed section: {0}")]
    CompressedSection(&'static str),
}

pub fn new_context<'data, O: Object<'data>>(file: &O) -> Result<Context<'data>, Error> {
    new_context_with_sup(file, None)
}

pub fn new_context_with_sup<'data, O: Object<'data>>(
    file: &O,
    sup_file: Option<&O>,
) -> Result<Context<'data>, Error> {
    let endian = match file.endianness() {
        object::Endianness::Little => gimli::RunTimeEndian::Little,
        object::Endianness::Big => gimli::RunTimeEndian::Big,
    };
    let mut dwarf = gimli::Dwarf::load(|id| load_section(id, file, endian))?;
    if let Some(sup_file) = sup_file {
        dwarf.load_sup(|id| load_section(id, sup_file, endian))?;
    }
    Ok(Context::from_dwarf(dwarf)?)
}

fn load_section<'data, O, Endian>(
    id: gimli::SectionId,
    file: &O,
    endian: Endian,
) -> Result<gimli::EndianSlice<'data, Endian>, Error>
where
    O: Object<'data>,
    Endian: gimli::Endianity,
{
    let data = match file.section_by_name(id.name()) {
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
}
