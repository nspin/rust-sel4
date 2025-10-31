//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;

use rkyv::tuple::ArchivedTuple2;

use crate::{ArchivedCap, ArchivedCapSlot, ArchivedCapTableEntry, TryFromCapError, cap, object};

// NOTE
// Magic constants must be kept in sync with capDL-tool.

pub trait HasArchivedCapTable {
    fn archived_slots(&self) -> &[ArchivedCapTableEntry];

    fn slots(&self) -> impl Iterator<Item = (&ArchivedCapSlot, &ArchivedCap)> {
        self.archived_slots()
            .iter()
            .map(|ArchivedTuple2(k, v)| (k, v))
    }

    fn maybe_slot(&self, slot: ArchivedCapSlot) -> Option<&ArchivedCap> {
        self.slots()
            .find_map(|(k, v)| if k == &slot { Some(v) } else { None })
    }

    fn maybe_slot_as<'a, T: TryFrom<&'a ArchivedCap>>(&'a self, slot: ArchivedCapSlot) -> Option<T>
    where
        <T as TryFrom<&'a ArchivedCap>>::Error: fmt::Debug,
    {
        self.maybe_slot(slot).map(|cap| cap.try_into().unwrap())
    }

    fn slot_as<'a, T: TryFrom<&'a ArchivedCap>>(&'a self, slot: ArchivedCapSlot) -> T
    where
        <T as TryFrom<&'a ArchivedCap>>::Error: fmt::Debug,
    {
        self.maybe_slot_as(slot).unwrap()
    }

    #[allow(clippy::type_complexity)]
    fn slots_as<'a, T: TryFrom<&'a ArchivedCap>>(
        &'a self,
    ) -> impl Iterator<Item = (ArchivedCapSlot, T)>
    where
        <T as TryFrom<&'a ArchivedCap>>::Error: fmt::Debug,
    {
        self.slots().map(|(k, v)| (*k, T::try_from(v).unwrap()))
    }
}

impl object::ArchivedTcb {
    pub const SLOT_CSPACE: ArchivedCapSlot = ArchivedCapSlot::from_native(0);
    pub const SLOT_VSPACE: ArchivedCapSlot = ArchivedCapSlot::from_native(1);
    pub const SLOT_IPC_BUFFER: ArchivedCapSlot = ArchivedCapSlot::from_native(4);
    pub const SLOT_FAULT_EP: ArchivedCapSlot = ArchivedCapSlot::from_native(5);
    pub const SLOT_SC: ArchivedCapSlot = ArchivedCapSlot::from_native(6);
    pub const SLOT_TEMP_FAULT_EP: ArchivedCapSlot = ArchivedCapSlot::from_native(7);
    pub const SLOT_BOUND_NOTIFICATION: ArchivedCapSlot = ArchivedCapSlot::from_native(8);
    pub const SLOT_VCPU: ArchivedCapSlot = ArchivedCapSlot::from_native(9);

    pub fn cspace(&self) -> &cap::ArchivedCNode {
        self.slot_as(Self::SLOT_CSPACE)
    }

    pub fn vspace(&self) -> &cap::ArchivedPageTable {
        self.slot_as(Self::SLOT_VSPACE)
    }

    pub fn ipc_buffer(&self) -> &cap::ArchivedFrame {
        self.slot_as(Self::SLOT_IPC_BUFFER)
    }

    pub fn mcs_fault_ep(&self) -> Option<&cap::ArchivedEndpoint> {
        self.maybe_slot_as(Self::SLOT_FAULT_EP)
    }

    pub fn sc(&self) -> Option<&cap::ArchivedSchedContext> {
        self.maybe_slot_as(Self::SLOT_SC)
    }

    pub fn temp_fault_ep(&self) -> Option<&cap::ArchivedEndpoint> {
        self.maybe_slot_as(Self::SLOT_TEMP_FAULT_EP)
    }

    pub fn bound_notification(&self) -> Option<&cap::ArchivedNotification> {
        self.maybe_slot_as(Self::SLOT_BOUND_NOTIFICATION)
    }

    pub fn vcpu(&self) -> Option<&cap::ArchivedVCpu> {
        self.maybe_slot_as(Self::SLOT_VCPU)
    }
}

impl object::ArchivedIrq {
    pub const SLOT_NOTIFICATION: ArchivedCapSlot = ArchivedCapSlot::from_native(0);

    pub fn notification(&self) -> Option<&cap::ArchivedNotification> {
        self.maybe_slot_as(Self::SLOT_NOTIFICATION)
    }
}

impl object::ArchivedArmIrq {
    pub const SLOT_NOTIFICATION: ArchivedCapSlot = ArchivedCapSlot::from_native(0);

    pub fn notification(&self) -> Option<&cap::ArchivedNotification> {
        self.maybe_slot_as(Self::SLOT_NOTIFICATION)
    }
}

impl object::ArchivedIrqMsi {
    pub const SLOT_NOTIFICATION: ArchivedCapSlot = ArchivedCapSlot::from_native(0);

    pub fn notification(&self) -> Option<&cap::ArchivedNotification> {
        self.maybe_slot_as(Self::SLOT_NOTIFICATION)
    }
}

impl object::ArchivedIrqIOApic {
    pub const SLOT_NOTIFICATION: ArchivedCapSlot = ArchivedCapSlot::from_native(0);

    pub fn notification(&self) -> Option<&cap::ArchivedNotification> {
        self.maybe_slot_as(Self::SLOT_NOTIFICATION)
    }
}

impl object::ArchivedRiscvIrq {
    pub const SLOT_NOTIFICATION: ArchivedCapSlot = ArchivedCapSlot::from_native(0);

    pub fn notification(&self) -> Option<&cap::ArchivedNotification> {
        self.maybe_slot_as(Self::SLOT_NOTIFICATION)
    }
}

// // // //

impl object::ArchivedPageTable {
    pub fn entries(&self) -> impl Iterator<Item = (ArchivedCapSlot, ArchivedPageTableEntry<'_>)> {
        self.slots_as()
    }

    pub fn frames(&self) -> impl Iterator<Item = (ArchivedCapSlot, &cap::ArchivedFrame)> {
        self.slots_as()
    }

    pub fn page_tables(&self) -> impl Iterator<Item = (ArchivedCapSlot, &cap::ArchivedPageTable)> {
        self.slots_as()
    }
}

pub enum ArchivedPageTableEntry<'a> {
    PageTable(&'a cap::ArchivedPageTable),
    Frame(&'a cap::ArchivedFrame),
}

impl<'a> ArchivedPageTableEntry<'a> {
    pub fn page_table(&self) -> Option<&'a cap::ArchivedPageTable> {
        match self {
            Self::PageTable(cap) => Some(cap),
            _ => None,
        }
    }

    pub fn frame(&self) -> Option<&'a cap::ArchivedFrame> {
        match self {
            Self::Frame(cap) => Some(cap),
            _ => None,
        }
    }
}

impl<'a> TryFrom<&'a ArchivedCap> for ArchivedPageTableEntry<'a> {
    type Error = TryFromCapError;

    fn try_from(cap: &'a ArchivedCap) -> Result<Self, Self::Error> {
        Ok(match cap {
            ArchivedCap::PageTable(cap) => ArchivedPageTableEntry::PageTable(cap),
            ArchivedCap::Frame(cap) => ArchivedPageTableEntry::Frame(cap),
            _ => return Err(TryFromCapError),
        })
    }
}
