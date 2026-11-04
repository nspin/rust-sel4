//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ffi::c_void;
use core::ops::Range;
use core::ptr::{self, NonNull};

use crate::config::impl_config;
use crate::sys;

unsafe fn device_resources_check_magic(config: *mut c_void) -> bool {
    unsafe { sys::device_resources_check_magic(config.cast()) }
}

impl_config! {
    device_resources_check_magic [
        sys::device_resources_t,
    ]
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct Region(sys::region_resource_t);

impl Region {
    pub fn from_resource(resource: &sys::region_resource_t) -> Option<&Self> {
        if resource.vaddr.is_null() {
            return None;
        }
        usize::try_from(resource.size).unwrap();
        // SAFETY: repr(transparent), and the invariant has just been checked
        Some(unsafe { &*ptr::from_ref(resource).cast::<Self>() })
    }

    pub fn as_raw(&self) -> &sys::region_resource_t {
        &self.0
    }

    pub fn vaddr(&self) -> NonNull<u8> {
        NonNull::new(self.0.vaddr.cast()).unwrap()
    }

    pub fn size(&self) -> usize {
        self.0.size.try_into().unwrap()
    }

    pub fn slice(&self) -> NonNull<[u8]> {
        NonNull::slice_from_raw_parts(self.vaddr(), self.size())
    }
}

/// A `device_region_resource_t` whose region is a [`Region`]. Its I/O address is the address at
/// which devices (or, more generally, peers which refer to buffers by I/O address) see it.
#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct DeviceRegion(sys::device_region_resource_t);

impl DeviceRegion {
    pub fn from_resource(resource: &sys::device_region_resource_t) -> Option<&Self> {
        Region::from_resource(&resource.region)?;
        // SAFETY: repr(transparent), and the invariant has just been checked
        Some(unsafe { &*ptr::from_ref(resource).cast::<Self>() })
    }

    pub fn as_raw(&self) -> &sys::device_region_resource_t {
        &self.0
    }

    pub fn region(&self) -> &Region {
        Region::from_resource(&self.0.region).unwrap()
    }

    pub fn io_range_to_offsets(&self, io_range: Range<usize>) -> Option<Range<usize>> {
        let start = io_range.start.checked_sub(self.0.io_addr)?;
        let end = io_range.end.checked_sub(self.0.io_addr)?;
        (start <= end && end <= self.region().size()).then_some(start..end)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DeviceResources {
    inner: sys::device_resources_t,
}

impl DeviceResources {
    pub fn new(inner: sys::device_resources_t) -> Self {
        assert!(usize::from(inner.num_regions) <= inner.regions.len());
        assert!(usize::from(inner.num_irqs) <= inner.irqs.len());
        for region in &inner.regions[..inner.num_regions.into()] {
            DeviceRegion::from_resource(region).unwrap();
        }
        Self { inner }
    }

    pub fn as_raw(&self) -> &sys::device_resources_t {
        &self.inner
    }

    pub fn region(&self, i: usize) -> &DeviceRegion {
        assert!(i < self.inner.num_regions.into());
        DeviceRegion::from_resource(&self.inner.regions[i]).unwrap()
    }

    pub fn regions(&self) -> impl Iterator<Item = &DeviceRegion> {
        (0..self.inner.num_regions.into()).map(|i| self.region(i))
    }

    pub fn irq(&self, i: usize) -> usize {
        assert!(i < self.inner.num_irqs.into());
        self.inner.irqs[i].id.into()
    }

    pub fn irqs(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.inner.num_irqs.into()).map(|i| self.irq(i))
    }
}
