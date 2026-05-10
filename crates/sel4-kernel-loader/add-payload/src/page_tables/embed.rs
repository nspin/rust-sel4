//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::collections::BTreeMap;

use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote};

use super::scheme::{Scheme, SchemeLeafDescriptor};
use super::table::{AbstractEntry, Table};

impl<T: Scheme> Table<T> {
    pub fn embed(&self, vaddr: u64) -> (Vec<T::WordPrimitive>, u64)
    where
        T::WordPrimitive: ToTokens,
    {
        Embedding::new(vaddr).embed(self)
    }
}

struct Embedding<T: Scheme> {
    vaddr: u64,
    buf: Vec<T::WordPrimitive>,
}

impl<T: Scheme> Embedding<T> {
    fn new(vaddr: u64) -> Self {
        Self {
            vaddr,
            buf: vec![],
        }
    }

    fn embed(mut self, table: &Table<T>) -> (Vec<T::WordPrimitive>, u64) {
        let vaddr = self.embed_inner(table, 0);
        (self.buf, vaddr)
    }

    fn embed_inner(&mut self, table: &Table<T>, level: usize) -> u64 {
        todo!()
        // let index = self.allocate_index();
        // let entries = table.entries.iter().map(|entry| {
        //     let entry = EntryForEmbedding::<T>::from_abstract_entry(entry);
        //     let ptr = match &entry.ptr {
        //         None => {
        //             quote! {
        //                 None
        //             }
        //         }
        //         Some(ptr) => {
        //             let child_index = self.embed_inner(ptr, level + 1);
        //             let symbol_access_ident = format_ident!("{}_access", self.symbol_ident);
        //             quote! {
        //                 Some(#symbol_access_ident.table(#child_index).value())
        //             }
        //         }
        //     };
        //     let offset = entry.offset;
        //     quote! {
        //         Entry::new(#ptr, #offset as usize)
        //     }
        // });
        // let align_type = format_ident!("A{}", 1usize << T::level_align_bits(level));
        // let num_entries = table.entries.len();
        // let toks = quote! {
        //     {
        //         static TABLE: Table<#align_type, #num_entries> = Table::new([#(#entries,)*]);
        //         TABLE.ptr()
        //     }
        // };
        // self.tables.insert(index, toks);
        // index
    }
}
