//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::{
    collections::BTreeMap,
    fs::File,
    os::unix::fs::FileExt as _,
    path::{Path, PathBuf},
};

use sel4_capdl_initializer_types::*;

use super::ObjectNamesLevel;

pub fn reserialize_spec(
    input_spec: &InputSpec,
    fill_dirs: &[impl AsRef<Path>],
    object_names_level: &ObjectNamesLevel,
    embed_frames: bool,
    granule_size_bits: u8,
) -> (SpecForInitializer, EmbeddedFramesData) {
    let granule_size = 1 << granule_size_bits;

    let mut filler = Filler::new(fill_dirs);

    let mut embedded_frames_data = EmbeddedFramesData::new(granule_size);

    let mut output_spec: SpecForInitializer = input_spec
        .split_embedded_frames(embed_frames, granule_size_bits)
        .traverse_data_with_length(|data, length| {
            let mut buf = vec![0; length.into_usize()];
            filler.read(data, &mut buf);
            DeflatedBytesContent::pack(&buf)
        })
        .traverse_embedded_frames(|fill| {
            let mut frame = vec![0; granule_size];
            for entry in fill.entries.iter() {
                filler.read(
                    entry.content.as_data().unwrap(),
                    &mut frame[u64::into_usize_range(&entry.range)],
                )
            }
            embedded_frames_data.align_to(granule_size);
            let start = embedded_frames_data.append(&frame);
            EmbeddedFrameOffset::new(start.try_into().unwrap())
        });

    for (name, obj) in output_spec.names_mut() {
        let keep = match object_names_level {
            ObjectNamesLevel::All => true,
            ObjectNamesLevel::JustTcbs => matches!(obj, Object::Tcb(_)),
            ObjectNamesLevel::None => false,
        };
        if !keep {
            *name = None;
        }
    }

    (output_spec, embedded_frames_data)
}

struct Filler {
    fill_dirs: Vec<PathBuf>,
    file_handles: BTreeMap<String, File>,
}

impl Filler {
    fn new(fill_dirs: impl IntoIterator<Item = impl AsRef<Path>>) -> Self {
        Self {
            fill_dirs: fill_dirs
                .into_iter()
                .map(|path| path.as_ref().to_owned())
                .collect(),
            file_handles: BTreeMap::new(),
        }
    }

    fn find_path(&self, file: &str) -> PathBuf {
        self.fill_dirs
            .iter()
            .filter_map(|dir| {
                let path = dir.join(file);
                if path.exists() { Some(path) } else { None }
            })
            .next()
            .unwrap_or_else(|| panic!("file {:?} not found", file))
    }

    fn get_handle(&mut self, file: &str) -> &mut File {
        let path = self.find_path(file);
        self.file_handles
            .entry(file.to_owned())
            .or_insert_with(|| File::open(path).unwrap())
    }

    fn read(&mut self, key: &FileContent, buf: &mut [u8]) {
        self.get_handle(&key.file)
            .read_exact_at(buf, key.file_offset)
            .unwrap();
    }
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
