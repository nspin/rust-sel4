//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use crate::{
    ArchivedCap, ArchivedCapSlot, ArchivedCapTableEntry, Cap, CapSlot, CapTableEntry,
    IsArchivedCap, IsCap, cap, object,
};

// NOTE
// Magic constants must be kept in sync with capDL-tool.

pub trait HasCapTable {
    fn slots(&self) -> &[CapTableEntry];

    fn maybe_slot(&self, slot: CapSlot) -> Option<&Cap> {
        self.slots().as_ref().iter().find_map(|entry| {
            if entry.slot == slot {
                Some(&entry.cap)
            } else {
                None
            }
        })
    }

    fn maybe_slot_as<T: IsCap>(&self, slot: CapSlot) -> Option<&T> {
        self.maybe_slot(slot).map(|cap| cap.as_().unwrap())
    }

    fn slot_as<T: IsCap>(&self, slot: CapSlot) -> &T {
        self.maybe_slot_as(slot).unwrap()
    }
}

pub trait HasArchivedCapTable {
    fn slots(&self) -> &[ArchivedCapTableEntry];

    fn maybe_slot(&self, slot: ArchivedCapSlot) -> Option<&ArchivedCap> {
        self.slots().as_ref().iter().find_map(|entry| {
            if entry.slot == slot {
                Some(&entry.cap)
            } else {
                None
            }
        })
    }

    fn maybe_slot_as<T: IsArchivedCap>(&self, slot: ArchivedCapSlot) -> Option<&T> {
        self.maybe_slot(slot).map(|cap| cap.as_().unwrap())
    }

    fn slot_as<T: IsArchivedCap>(&self, slot: ArchivedCapSlot) -> &T {
        self.maybe_slot_as(slot).unwrap()
    }
}

trait SlotTarget<'a>: 'a {
    fn get_slot_target<U: HasCapTable>(table: &'a U, slot: CapSlot) -> Self;
}

impl<'a, T: IsCap> SlotTarget<'a> for &'a T {
    fn get_slot_target<U: HasCapTable>(table: &'a U, slot: CapSlot) -> Self {
        table.slot_as(slot)
    }
}

impl<'a, T: IsCap> SlotTarget<'a> for Option<&'a T> {
    fn get_slot_target<U: HasCapTable>(table: &'a U, slot: CapSlot) -> Self {
        table.maybe_slot_as(slot)
    }
}

trait SlotTargetArchived<'a>: 'a {
    fn get_slot_target_archived<U: HasArchivedCapTable>(
        table: &'a U,
        slot: ArchivedCapSlot,
    ) -> Self;
}

impl<'a, T: IsArchivedCap> SlotTargetArchived<'a> for &'a T {
    fn get_slot_target_archived<U: HasArchivedCapTable>(
        table: &'a U,
        slot: ArchivedCapSlot,
    ) -> Self {
        table.slot_as(slot)
    }
}

impl<'a, T: IsArchivedCap> SlotTargetArchived<'a> for Option<&'a T> {
    fn get_slot_target_archived<U: HasArchivedCapTable>(
        table: &'a U,
        slot: ArchivedCapSlot,
    ) -> Self {
        table.maybe_slot_as(slot)
    }
}

macro_rules! alias_cap_table {
    ($obj_ty:ty | $archived_obj_ty:ty {
        $(
            $accessor_name:ident: $ty:ty | $archived_ty:ty = $slot_name:ident($n:expr)
        ),* $(,)?
    }) => {
        impl $obj_ty {
            $(
                pub const $slot_name: CapSlot = $n;

                pub fn $accessor_name(&self) -> $ty {
                    <$ty>::get_slot_target(self, Self::$slot_name)
                }
            )*
        }

        impl $archived_obj_ty {
            $(
                pub const $slot_name: ArchivedCapSlot = ArchivedCapSlot::from_native($n);

                pub fn $accessor_name(&self) -> $archived_ty {
                    <$archived_ty>::get_slot_target_archived(self, Self::$slot_name)
                }
            )*
        }
    };
}

alias_cap_table! {
    object::Tcb | object::ArchivedTcb {
        cspace: &cap::CNode | &cap::ArchivedCNode = SLOT_CSPACE(0),
        vspace: &cap::PageTable | &cap::ArchivedPageTable = SLOT_VSPACE(1),
        ipc_buffer: &cap::Frame | &cap::ArchivedFrame = SLOT_IPC_BUFFER(4),
        mcs_fault_ep: Option<&cap::Endpoint> | Option<&cap::ArchivedEndpoint> = SLOT_FAULT_EP(5),
        sc: Option<&cap::SchedContext> | Option<&cap::ArchivedSchedContext> = SLOT_SC(6),
        temp_fault_ep: Option<&cap::Endpoint> | Option<&cap::ArchivedEndpoint> = SLOT_TEMP_FAULT_EP(7),
        bound_notification: Option<&cap::Notification> | Option<&cap::ArchivedNotification> = SLOT_BOUND_NOTIFICATION(8),
        vcpu: Option<&cap::VCpu> | Option<&cap::ArchivedVCpu> = SLOT_VCPU(9),
    }
}

alias_cap_table! {
    object::Irq | object::ArchivedIrq {
        notification: Option<&cap::Notification> | Option<&cap::ArchivedNotification> = SLOT_NOTIFICATION(0),
    }
}

alias_cap_table! {
    object::ArmIrq | object::ArchivedArmIrq {
        notification: Option<&cap::Notification> | Option<&cap::ArchivedNotification> = SLOT_NOTIFICATION(0),
    }
}

alias_cap_table! {
    object::IrqMsi | object::ArchivedIrqMsi {
        notification: Option<&cap::Notification> | Option<&cap::ArchivedNotification> = SLOT_NOTIFICATION(0),
    }
}

alias_cap_table! {
    object::IrqIOApic | object::ArchivedIrqIOApic {
        notification: Option<&cap::Notification> | Option<&cap::ArchivedNotification> = SLOT_NOTIFICATION(0),
    }
}

alias_cap_table! {
    object::RiscvIrq | object::ArchivedRiscvIrq {
        notification: Option<&cap::Notification> | Option<&cap::ArchivedNotification> = SLOT_NOTIFICATION(0),
    }
}
