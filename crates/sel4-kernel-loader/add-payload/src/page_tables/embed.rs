//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use super::scheme::{Scheme, SchemeLeafDescriptor};
use super::table::{AbstractEntry, Table};

impl<T: Scheme> Table<T> {
    pub fn embed(&self, vaddr: u64) -> (Vec<T::WordPrimitive>, u64) {
        Embedding::new(vaddr).embed(self)
    }
}

struct Embedding<T: Scheme> {
    start_vaddr: u64,
    buf: Vec<T::WordPrimitive>,
}

impl<T: Scheme> Embedding<T> {
    fn new(start_vaddr: u64) -> Self {
        Self {
            start_vaddr,
            buf: vec![],
        }
    }

    fn embed(mut self, table: &Table<T>) -> (Vec<T::WordPrimitive>, u64) {
        let root_vaddr = self.embed_inner(table, 0);
        (self.buf, root_vaddr)
    }

    fn embed_inner(&mut self, table: &Table<T>, level: usize) -> u64 {
        let entries = table
            .entries
            .iter()
            .map(|entry| match entry {
                AbstractEntry::Empty => T::EMPTY_DESCRIPTOR,
                AbstractEntry::Leaf(descriptor) => descriptor.to_raw(),
                AbstractEntry::Branch(branch) => {
                    let child_vaddr = self.embed_inner(branch, level + 1);
                    T::mk_branch_descriptor(child_vaddr)
                }
            })
            .collect::<Vec<_>>();
        self.align(1 << T::level_align_bits(level));
        let vaddr = self.cur_vaddr();
        self.buf.extend(entries);
        vaddr
    }

    fn cur_vaddr(&self) -> u64 {
        self.start_vaddr + Self::word_bytes() * u64::try_from(self.buf.len()).unwrap()
    }

    fn align(&mut self, align: u64) {
        let cur_vaddr = self.cur_vaddr();
        let aligned_vaddr = cur_vaddr.next_multiple_of(align);
        self.buf.resize_with(
            (aligned_vaddr - self.start_vaddr).try_into().unwrap(),
            || T::EMPTY_DESCRIPTOR,
        );
    }

    fn word_bytes() -> u64 {
        size_of::<T::WordPrimitive>().try_into().unwrap()
    }
}
