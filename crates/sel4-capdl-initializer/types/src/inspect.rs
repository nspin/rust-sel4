//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::convert::Infallible;

use crate::{CramUsize, NamedObject, Object, ObjectId, Spec, object};

impl<D> Spec<D> {
    pub fn traverse_frame_init_fallible<D1, E>(
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

    pub fn traverse_frame_init<D1>(
        &self,
        mut f: impl FnMut(&object::Frame<D>, bool) -> D1,
    ) -> Spec<D1> {
        self.traverse_frame_init_fallible(|x1, x2| Ok(f(x1, x2)))
            .unwrap_or_else(|absurdity: Infallible| match absurdity {})
    }
}
