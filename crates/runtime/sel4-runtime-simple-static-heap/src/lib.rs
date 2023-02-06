#![no_std]
#![feature(const_option_ext)]
#![feature(strict_provenance)]

use core::ops::Range;

use sel4_dlmalloc::{StaticDlmallocGlobalAlloc, StaticHeap};
use sel4_env_literal_helper::env_literal;
use sel4_sync::{DeferredNotificationMutexSyncOps, MutexSyncOpsWithInteriorMutability};

const STATIC_HEAP_SIZE: usize = env_literal!("SEL4_RUNTIME_HEAP_SIZE").unwrap_or(0);

static mut STATIC_HEAP: StaticHeap<STATIC_HEAP_SIZE> = StaticHeap::new();

#[global_allocator]
static GLOBAL_ALLOCATOR: StaticDlmallocGlobalAlloc<
    DeferredNotificationMutexSyncOps,
    fn() -> Range<*mut u8>,
> = StaticDlmallocGlobalAlloc::new(DeferredNotificationMutexSyncOps::new(), || unsafe {
    STATIC_HEAP.bounds()
});

pub fn set_mutex_notification(
    notification: <DeferredNotificationMutexSyncOps as MutexSyncOpsWithInteriorMutability>::ModifyInput,
) {
    GLOBAL_ALLOCATOR.mutex().modify_sync_ops(notification)
}
