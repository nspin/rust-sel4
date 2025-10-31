//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::fmt;
use core::ops::Range;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use sel4_capdl_initializer_types_derive::{IsCap, IsObject, IsObjectWithCapTable};

use crate::{FrameInit, HasCapTable};

pub type PortableWord = u64;
pub type PortableBadge = PortableWord;
pub type PortableCPtr = PortableWord;

pub type ObjectId = u32;

pub type CapSlot = u32;
pub type CapTableEntry = (CapSlot, Cap);

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct Spec<N, D, M> {
    pub objects: Vec<NamedObject<N, D, M>>,
    pub irqs: Vec<IrqEntry>,
    pub asid_slots: Vec<AsidSlotEntry>,
    pub root_objects: Range<ObjectId>,
    pub untyped_covers: Vec<UntypedCover>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct IrqEntry {
    pub irq: PortableWord,
    pub handler: ObjectId,
}

pub type AsidSlotEntry = ObjectId;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct UntypedCover {
    pub parent: ObjectId,
    pub children: Range<ObjectId>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct NamedObject<N, D, M> {
    pub name: N,
    pub object: Object<D, M>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub enum Object<D, M> {
    Untyped(object::Untyped),
    Endpoint,
    Notification,
    CNode(object::CNode),
    Tcb(object::Tcb),
    Irq(object::Irq),
    VCpu,
    Frame(object::Frame<D, M>),
    PageTable(object::PageTable),
    AsidPool(object::AsidPool),
    ArmIrq(object::ArmIrq),
    IrqMsi(object::IrqMsi),
    IrqIOApic(object::IrqIOApic),
    RiscvIrq(object::RiscvIrq),
    IOPorts(object::IOPorts),
    SchedContext(object::SchedContext),
    Reply,
    ArmSmc,
}

impl<D, M> Object<D, M> {
    pub fn paddr(&self) -> Option<u64> {
        match self {
            Object::Untyped(obj) => obj.paddr,
            Object::Frame(obj) => obj.paddr,
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub enum Cap {
    Untyped(cap::Untyped),
    Endpoint(cap::Endpoint),
    Notification(cap::Notification),
    CNode(cap::CNode),
    Tcb(cap::Tcb),
    IrqHandler(cap::IrqHandler),
    VCpu(cap::VCpu),
    Frame(cap::Frame),
    PageTable(cap::PageTable),
    AsidPool(cap::AsidPool),
    ArmIrqHandler(cap::ArmIrqHandler),
    IrqMsiHandler(cap::IrqMsiHandler),
    IrqIOApicHandler(cap::IrqIOApicHandler),
    RiscvIrqHandler(cap::RiscvIrqHandler),
    IOPorts(cap::IOPorts),
    SchedContext(cap::SchedContext),
    Reply(cap::Reply),
    ArmSmc(cap::ArmSmc),
}

impl Cap {
    pub fn obj(&self) -> ObjectId {
        match self {
            Cap::Untyped(cap) => cap.object,
            Cap::Endpoint(cap) => cap.object,
            Cap::Notification(cap) => cap.object,
            Cap::CNode(cap) => cap.object,
            Cap::Frame(cap) => cap.object,
            Cap::Tcb(cap) => cap.object,
            Cap::IrqHandler(cap) => cap.object,
            Cap::VCpu(cap) => cap.object,
            Cap::PageTable(cap) => cap.object,
            Cap::AsidPool(cap) => cap.object,
            Cap::ArmIrqHandler(cap) => cap.object,
            Cap::IrqMsiHandler(cap) => cap.object,
            Cap::IrqIOApicHandler(cap) => cap.object,
            Cap::RiscvIrqHandler(cap) => cap.object,
            Cap::IOPorts(cap) => cap.object,
            Cap::SchedContext(cap) => cap.object,
            Cap::Reply(cap) => cap.object,
            Cap::ArmSmc(cap) => cap.object,
        }
    }
}

// TODO Would packing have an actual effect on memory footprint?
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct Rights {
    pub read: bool,
    pub write: bool,
    pub grant: bool,
    pub grant_reply: bool,
}

pub mod object {
    use super::*;

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct Untyped {
        pub size_bits: u8,
        pub paddr: Option<u64>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, IsObjectWithCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct CNode {
        pub size_bits: u8,
        pub slots: Vec<CapTableEntry>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, IsObjectWithCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct Tcb {
        pub slots: Vec<CapTableEntry>,
        pub extra: Box<TcbExtraInfo>,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct TcbExtraInfo {
        pub ipc_buffer_addr: PortableWord,

        pub affinity: PortableWord,
        pub prio: u8,
        pub max_prio: u8,
        pub resume: bool,

        pub ip: PortableWord,
        pub sp: PortableWord,
        pub gprs: Vec<PortableWord>,

        pub master_fault_ep: Option<PortableCPtr>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, IsObjectWithCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct Irq {
        pub slots: Vec<CapTableEntry>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct Frame<D, M> {
        pub size_bits: u8,
        pub paddr: Option<u64>,
        pub init: FrameInit<D, M>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, IsObjectWithCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct PageTable {
        pub is_root: bool,
        pub level: Option<u8>,
        pub slots: Vec<CapTableEntry>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct AsidPool {
        pub high: PortableWord,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, IsObjectWithCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct ArmIrq {
        pub slots: Vec<CapTableEntry>,
        pub extra: Box<ArmIrqExtraInfo>,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct ArmIrqExtraInfo {
        pub trigger: PortableWord,
        pub target: PortableWord,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, IsObjectWithCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct IrqMsi {
        pub slots: Vec<CapTableEntry>,
        pub extra: Box<IrqMsiExtraInfo>,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct IrqMsiExtraInfo {
        pub handle: PortableWord,
        pub pci_bus: PortableWord,
        pub pci_dev: PortableWord,
        pub pci_func: PortableWord,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, IsObjectWithCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct IrqIOApic {
        pub slots: Vec<CapTableEntry>,
        pub extra: Box<IrqIOApicExtraInfo>,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct IrqIOApicExtraInfo {
        pub ioapic: PortableWord,
        pub pin: PortableWord,
        pub level: PortableWord,
        pub polarity: PortableWord,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject, IsObjectWithCapTable)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct RiscvIrq {
        pub slots: Vec<CapTableEntry>,
        pub extra: RiscvIrqExtraInfo,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct RiscvIrqExtraInfo {
        pub trigger: PortableWord,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct IOPorts {
        pub start_port: PortableWord,
        pub end_port: PortableWord,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsObject)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct SchedContext {
        pub size_bits: u8,
        pub extra: SchedContextExtraInfo,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct SchedContextExtraInfo {
        pub period: u64,
        pub budget: u64,
        pub badge: PortableBadge,
    }
}

pub mod cap {
    use super::*;

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct Untyped {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct Endpoint {
        pub object: ObjectId,
        // TODO
        //   parse-capDL uses badge=0 to mean no badge. Is that good
        //   enough, or do we ever need to actually use the badge value '0'?
        // TODO
        //   Is it correct that these are ignored in the case of Tcb::SLOT_TEMP_FAULT_EP?
        pub badge: PortableBadge,
        pub rights: Rights,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct Notification {
        pub object: ObjectId,
        pub badge: PortableBadge,
        pub rights: Rights,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct CNode {
        pub object: ObjectId,
        pub guard: PortableWord,
        pub guard_size: PortableWord,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct Tcb {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct IrqHandler {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct VCpu {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct Frame {
        pub object: ObjectId,
        pub rights: Rights,
        pub cached: bool,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct PageTable {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct AsidPool {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct ArmIrqHandler {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct IrqMsiHandler {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct IrqIOApicHandler {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct RiscvIrqHandler {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct IOPorts {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct SchedContext {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct Reply {
        pub object: ObjectId,
    }

    #[derive(Debug, Clone, Eq, PartialEq, IsCap)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
    pub struct ArmSmc {
        pub object: ObjectId,
    }
}

// // //

#[derive(Debug)]
pub struct TryFromObjectError;

#[derive(Debug)]
pub struct TryFromCapError;

impl fmt::Display for TryFromObjectError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "object type mismatch")
    }
}

impl fmt::Display for TryFromCapError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "object type mismatch")
    }
}
