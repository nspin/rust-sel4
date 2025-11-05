//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::vec;
use alloc::vec::Vec;
use core::convert::Infallible;
use core::ops::Range;

use crate::{
    BytesContent, Content, CramUsize, DeflatedBytesContent, EmbeddedFrameIndex, Fill, FillEntry,
    FillEntryContent, FrameInit, NamedObject, Object, ObjectId, Spec, SpecForInitializer, object,
};

impl<D> Spec<Fill<D>> {
    pub fn embed_fill_fallible<E>(
        &self,
        embed_frames: bool,
        granule_size_bits: u8,
        mut f: impl FnMut(&D, &mut [u8]) -> Result<bool, E>,
    ) -> Result<(SpecForInitializer, Vec<Vec<u8>>), E> {
        let granule_size = 1 << granule_size_bits;
        let mut frame_data = vec![];
        let spec = self.traverse_frame_init_fallible(|frame, is_root| {
            Ok(
                if embed_frames && frame.can_embed(granule_size_bits, is_root) {
                    FrameInit::Embedded({
                        let mut frame_buf = vec![0; granule_size];
                        for entry in frame.init.entries.iter() {
                            f(
                                entry.content.as_data().unwrap(),
                                &mut frame_buf[u64::into_usize_range(&entry.range)],
                            )?;
                        }
                        let ix = frame_data.len();
                        frame_data.push(frame_buf);
                        EmbeddedFrameIndex {
                            index: ix.try_into().unwrap(),
                        }
                    })
                } else {
                    FrameInit::Fill({
                        frame.init.traverse_fallible(|range, data| {
                            let length = (range.end - range.start).try_into().unwrap();
                            let mut buf = vec![0; length];
                            let deflate = f(data, &mut buf)?;
                            Ok(if deflate {
                                Content::DeflatedBytes(DeflatedBytesContent::pack(&buf))
                            } else {
                                Content::Bytes(BytesContent::pack(&buf))
                            })
                        })?
                    })
                },
            )
        })?;
        Ok((spec, frame_data))
    }

    pub fn embed_fill(
        &self,
        embed_frames: bool,
        granule_size_bits: u8,
        mut f: impl FnMut(&D, &mut [u8]) -> bool,
    ) -> (SpecForInitializer, Vec<Vec<u8>>) {
        self.embed_fill_fallible(embed_frames, granule_size_bits, |x1, x2| Ok(f(x1, x2)))
            .unwrap_or_else(|absurdity: Infallible| match absurdity {})
    }
}

impl<D> Spec<D> {
    fn traverse_frame_init_fallible<D1, E>(
        &self,
        mut f: impl FnMut(&object::Frame<D>, bool) -> Result<D1, E>,
    ) -> Result<Spec<D1>, E> {
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
                                init: {
                                    let is_root = ObjectId::into_usize_range(&self.root_objects)
                                        .contains(&obj_id);
                                    f(obj, is_root)?
                                },
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

    #[allow(unused)]
    fn traverse_frame_init<D1>(
        &self,
        mut f: impl FnMut(&object::Frame<D>, bool) -> D1,
    ) -> Spec<D1> {
        self.traverse_frame_init_fallible(|x1, x2| Ok(f(x1, x2)))
            .unwrap_or_else(|absurdity: Infallible| match absurdity {})
    }
}

impl<D> object::Frame<Fill<D>> {
    pub(crate) fn can_embed(&self, granule_size_bits: u8, is_root: bool) -> bool {
        is_root
            && self.paddr.is_none()
            && self.size_bits == granule_size_bits
            && !self.init.is_empty()
            && !self.init.depends_on_bootinfo()
    }
}

impl<D> Fill<D> {
    fn depends_on_bootinfo(&self) -> bool {
        self.entries.iter().any(|entry| entry.content.is_bootinfo())
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn traverse_fallible<D1, E>(
        &self,
        mut f: impl FnMut(&Range<u64>, &D) -> Result<D1, E>,
    ) -> Result<Fill<D1>, E> {
        Ok(Fill {
            entries: self
                .entries
                .iter()
                .map(|entry| {
                    Ok(FillEntry {
                        range: entry.range.clone(),
                        content: match &entry.content {
                            FillEntryContent::BootInfo(content_bootinfo) => {
                                FillEntryContent::BootInfo(*content_bootinfo)
                            }
                            FillEntryContent::Data(content_data) => {
                                FillEntryContent::Data(f(&entry.range, content_data)?)
                            }
                        },
                    })
                })
                .collect::<Result<_, E>>()?,
        })
    }

    pub fn traverse<D1>(&self, mut f: impl FnMut(&Range<u64>, &D) -> D1) -> Fill<D1> {
        self.traverse_fallible(|x1, x2| Ok(f(x1, x2)))
            .unwrap_or_else(|absurdity: Infallible| match absurdity {})
    }
}
