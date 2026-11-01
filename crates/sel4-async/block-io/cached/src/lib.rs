//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::num::NonZeroUsize;

use futures::future;
use lru::LruCache;

use sel4_async_block_io::access::ReadOnly;
use sel4_async_block_io::{BlockIO, BlockIOLayout, BlockSize, Operation, PartialBlockIO};

#[derive(Debug)]
pub struct CachedBlockIO<T: BlockIOLayout> {
    inner: T,
    lru: RefCell<LruCache<u64, Vec<u8>>>,
}

impl<T: BlockIOLayout> CachedBlockIO<T> {
    pub fn new(inner: T, cache_size_in_blocks: usize) -> Self {
        Self {
            inner,
            lru: RefCell::new(LruCache::new(
                NonZeroUsize::new(cache_size_in_blocks).unwrap(),
            )),
        }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub const fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T: BlockIOLayout> BlockIOLayout for CachedBlockIO<T> {
    type Error = <T as BlockIOLayout>::Error;

    type BlockSize = <T as BlockIOLayout>::BlockSize;

    fn block_size(&self) -> Self::BlockSize {
        <T as BlockIOLayout>::block_size(self.inner())
    }

    fn num_blocks(&self) -> u64 {
        <T as BlockIOLayout>::num_blocks(self.inner())
    }
}

impl<T: BlockIO<ReadOnly>> BlockIO<ReadOnly> for CachedBlockIO<T> {
    async fn read_or_write_blocks(
        &self,
        start_block_idx: u64,
        operation: Operation<'_, ReadOnly>,
    ) -> Result<(), Self::Error> {
        let Operation::Read { buf, witness } = operation;
        let block_size = self.block_size().bytes();
        assert_eq!(buf.len() % block_size, 0);
        let num_blocks = buf.len() / block_size;

        let block_idx_at_offset_from =
            |start: u64, offset: usize| start.checked_add(offset.try_into().unwrap()).unwrap();

        let mut misses = Vec::new();
        {
            let mut rest = buf;
            let mut lru = self.lru.borrow_mut();
            let mut cur_run_start = None;
            for i in 0..num_blocks {
                let block_idx = block_idx_at_offset_from(start_block_idx, i);
                match lru.get(&block_idx) {
                    Some(block) => {
                        if let Some(run_start) = cur_run_start.take() {
                            misses.push((
                                run_start,
                                rest.split_off_mut(..(i - run_start) * block_size).unwrap(),
                            ));
                        }
                        rest.split_off_mut(..block_size)
                            .unwrap()
                            .copy_from_slice(block);
                    }
                    None => {
                        cur_run_start.get_or_insert(i);
                    }
                }
            }
            if let Some(run_start) = cur_run_start {
                misses.push((run_start, rest));
            }
        }

        future::try_join_all(misses.into_iter().map(|(run_start, run_buf)| async move {
            let run_start_block_idx = block_idx_at_offset_from(start_block_idx, run_start);
            self.inner
                .read_or_write_blocks(
                    run_start_block_idx,
                    Operation::Read {
                        buf: run_buf,
                        witness,
                    },
                )
                .await?;
            let mut lru = self.lru.borrow_mut();
            for (i, block_buf) in run_buf.chunks(block_size).enumerate() {
                let block_idx = block_idx_at_offset_from(run_start_block_idx, i);
                let _ = lru.put(block_idx, block_buf.to_vec());
            }
            Ok(())
        }))
        .await?;
        Ok(())
    }
}

impl<T: BlockIO<ReadOnly>> PartialBlockIO<ReadOnly> for CachedBlockIO<T> {
    async fn read_or_write_block_at_offset(
        &self,
        block_idx: u64,
        offset: usize,
        operation: Operation<'_, ReadOnly>,
    ) -> Result<(), Self::Error> {
        let Operation::Read { buf, .. } = operation;
        let range = offset..offset + buf.len();
        if let Some(block) = self.lru.borrow_mut().get(&block_idx) {
            buf.copy_from_slice(&block[range]);
        } else {
            let mut block = vec![0; self.block_size().bytes()];
            self.inner().read_blocks(block_idx, &mut block).await?;
            buf.copy_from_slice(&block[range]);
            let _ = self.lru.borrow_mut().put(block_idx, block);
        }
        Ok(())
    }
}
