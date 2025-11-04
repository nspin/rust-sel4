//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::string::String;
use core::convert::Infallible;

use crate::{
    CramUsize, Fill, FillEntry, FillEntryContent, FrameInit, NamedObject, NeverEmbedded, Object,
    ObjectId, Spec, object,
};

impl<D, M> Spec<D, M> {
    pub fn names_mut(&mut self) -> impl Iterator<Item = (&mut Option<String>, &Object<D, M>)> {
        self.objects
            .iter_mut()
            .map(|named_obj| (&mut named_obj.name, &named_obj.object))
    }

    pub(crate) fn traverse_frame_init<D1, M1, E>(
        &self,
        mut f: impl FnMut(&object::Frame<D, M>, bool) -> Result<FrameInit<D1, M1>, E>,
    ) -> Result<Spec<D1, M1>, E> {
        Ok(Spec {
            objects: self
                .objects
                .iter()
                .enumerate()
                .map(|(obj_id, named_obj)| {
                    Ok(NamedObject {
                        name: named_obj.name.clone(),
                        object: match &named_obj.object {
                            Object::Untyped(obj) => Object::Untyped(obj.clone()),
                            Object::Endpoint => Object::Endpoint,
                            Object::Notification => Object::Notification,
                            Object::CNode(obj) => Object::CNode(obj.clone()),
                            Object::Tcb(obj) => Object::Tcb(obj.clone()),
                            Object::Irq(obj) => Object::Irq(obj.clone()),
                            Object::VCpu => Object::VCpu,
                            Object::Frame(obj) => Object::Frame(object::Frame {
                                size_bits: obj.size_bits,
                                paddr: obj.paddr,
                                init: f(
                                    obj,
                                    ObjectId::into_usize_range(&self.root_objects)
                                        .contains(&obj_id),
                                )?,
                            }),
                            Object::PageTable(obj) => Object::PageTable(obj.clone()),
                            Object::AsidPool(obj) => Object::AsidPool(obj.clone()),
                            Object::ArmIrq(obj) => Object::ArmIrq(obj.clone()),
                            Object::IrqMsi(obj) => Object::IrqMsi(obj.clone()),
                            Object::IrqIOApic(obj) => Object::IrqIOApic(obj.clone()),
                            Object::RiscvIrq(obj) => Object::RiscvIrq(obj.clone()),
                            Object::IOPorts(obj) => Object::IOPorts(obj.clone()),
                            Object::SchedContext(obj) => Object::SchedContext(obj.clone()),
                            Object::Reply => Object::Reply,
                            Object::ArmSmc => Object::ArmSmc,
                        },
                    })
                })
                .collect::<Result<_, E>>()?,
            irqs: self.irqs.clone(),
            asid_slots: self.asid_slots.clone(),
            root_objects: self.root_objects.clone(),
            untyped_covers: self.untyped_covers.clone(),
        })
    }
}

impl<D, M: Clone> Spec<D, M> {
    pub fn traverse_data_with_length_fallible<D1, E>(
        &self,
        mut f: impl FnMut(&D, u64) -> Result<D1, E>,
    ) -> Result<Spec<D1, M>, E> {
        self.traverse_frame_init(|frame, _is_root| {
            Ok(match &frame.init {
                FrameInit::Fill(fill) => FrameInit::Fill(Fill {
                    entries: fill
                        .entries
                        .iter()
                        .map(|entry| {
                            Ok(FillEntry {
                                range: entry.range.clone(),
                                content: match &entry.content {
                                    FillEntryContent::BootInfo(content_bootinfo) => {
                                        FillEntryContent::BootInfo(*content_bootinfo)
                                    }
                                    FillEntryContent::Data(content_data) => FillEntryContent::Data(
                                        f(content_data, entry.range.end - entry.range.start)?,
                                    ),
                                },
                            })
                        })
                        .collect::<Result<_, E>>()?,
                }),
                FrameInit::Embedded(embedded) => FrameInit::Embedded(embedded.clone()),
            })
        })
    }

    pub fn traverse_data_with_length<D1>(&self, mut f: impl FnMut(&D, u64) -> D1) -> Spec<D1, M> {
        unwrap_infallible(self.traverse_data_with_length_fallible(|x1, x2| Ok(f(x1, x2))))
    }

    pub fn traverse_data_fallible<D1, E>(
        &self,
        mut f: impl FnMut(&D) -> Result<D1, E>,
    ) -> Result<Spec<D1, M>, E> {
        self.traverse_data_with_length_fallible(|data, _length| f(data))
    }

    pub fn traverse_data<D1>(&self, mut f: impl FnMut(&D) -> D1) -> Spec<D1, M> {
        unwrap_infallible(self.traverse_data_fallible(|x| Ok(f(x))))
    }
}

impl<D: Clone, M> Spec<D, M> {
    pub fn traverse_embedded_frames_fallible<M1, E>(
        &self,
        mut f: impl FnMut(&M) -> Result<M1, E>,
    ) -> Result<Spec<D, M1>, E> {
        self.traverse_frame_init(|frame, _is_root| {
            Ok(match &frame.init {
                FrameInit::Fill(fill) => FrameInit::Fill(fill.clone()),
                FrameInit::Embedded(embedded) => FrameInit::Embedded(f(embedded)?),
            })
        })
    }

    pub fn traverse_embedded_frames<M1>(&self, mut f: impl FnMut(&M) -> M1) -> Spec<D, M1> {
        unwrap_infallible(self.traverse_embedded_frames_fallible(|x| Ok(f(x))))
    }
}

impl<D: Clone> Spec<D, NeverEmbedded> {
    pub fn split_embedded_frames(
        &self,
        embed_frames: bool,
        granule_size_bits: u8,
    ) -> Spec<D, Fill<D>> {
        unwrap_infallible(
            self.traverse_frame_init::<_, _, Infallible>(|frame, is_root| {
                let fill = frame.init.as_fill_infallible();
                Ok(
                    if embed_frames && frame.can_embed(granule_size_bits, is_root) {
                        FrameInit::Embedded(fill.clone())
                    } else {
                        FrameInit::Fill(fill.clone())
                    },
                )
            }),
        )
    }
}

fn unwrap_infallible<T>(result: Result<T, Infallible>) -> T {
    result.unwrap_or_else(|absurdity| match absurdity {})
}
