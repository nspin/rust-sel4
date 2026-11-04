//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::cell::RefCell;

use async_unsync::semaphore::Semaphore;
use heapless::{Deque, Vec};

/// IDs are reused in FIFO order: the ID which was freed longest ago is handed out next. This makes
/// it less likely that a stale or duplicate response from the peer matches a newer request. sDDF's
/// `ialloc` behaves the same way.
pub(crate) struct RequestTable<T, const N: usize> {
    free_id_permits: Semaphore,
    inner: RefCell<Inner<T, N>>,
}

struct Inner<T, const N: usize> {
    entries: Vec<Option<T>, N>,
    free_ids: Deque<usize, N>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum LookupError {
    OutOfRange,
    NotInUse,
}

impl<T, const N: usize> RequestTable<T, N> {
    /// Panics if `capacity > N`.
    pub(crate) fn new(capacity: usize) -> Self {
        assert!(capacity <= N);
        let mut entries = Vec::new();
        entries.extend((0..capacity).map(|_| None));
        let mut free_ids = Deque::new();
        for id in 0..capacity {
            free_ids.push_back(id).unwrap();
        }
        Self {
            free_id_permits: Semaphore::new(capacity),
            inner: RefCell::new(Inner { entries, free_ids }),
        }
    }

    pub(crate) async fn insert(&self, value: T) -> usize {
        // The permit stands for the ID until it is freed
        self.free_id_permits.acquire().await.unwrap().forget();
        let mut inner = self.inner.borrow_mut();
        let id = inner.free_ids.pop_front().unwrap();
        inner.entries[id] = Some(value);
        id
    }

    /// Takes `id`'s value, passes it to `f`, and stores the value that `f` returns, or frees `id`
    /// if `f` returns `None`. Returns `f`'s output.
    ///
    /// `f` must not use the table.
    pub(crate) fn update<R>(
        &self,
        id: usize,
        f: impl FnOnce(T) -> (Option<T>, R),
    ) -> Result<R, LookupError> {
        let mut inner = self.inner.borrow_mut();
        let entry = inner.entry_mut(id)?;
        let value = entry.take().ok_or(LookupError::NotInUse)?;
        let (new_value, output) = f(value);
        let freed = new_value.is_none();
        *entry = new_value;
        if freed {
            inner.free_ids.push_back(id).unwrap();
            self.free_id_permits.add_permits(1);
        }
        Ok(output)
    }
}

impl<T, const N: usize> Inner<T, N> {
    fn entry_mut(&mut self, id: usize) -> Result<&mut Option<T>, LookupError> {
        self.entries.get_mut(id).ok_or(LookupError::OutOfRange)
    }
}
