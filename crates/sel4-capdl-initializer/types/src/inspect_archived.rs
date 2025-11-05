//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ops::Range;

use rkyv::Archive;
use rkyv::ops::ArchivedRange;
use rkyv::option::ArchivedOption;
use rkyv::string::ArchivedString;

use crate::{
    ArchivedNamedObject, ArchivedObject, ArchivedObjectId, ArchivedSpec, CramUsize,
    TryFromObjectError, spec::IsArchivedObject,
};

impl<D: Archive> ArchivedSpec<D> {
    pub fn num_objects(&self) -> usize {
        self.objects.len()
    }

    pub fn named_object(&self, obj_id: ArchivedObjectId) -> &ArchivedNamedObject<D> {
        &self.objects[obj_id.into_usize()]
    }

    pub fn name(&self, obj_id: ArchivedObjectId) -> &ArchivedOption<ArchivedString> {
        &self.named_object(obj_id).name
    }

    pub fn object(&self, obj_id: ArchivedObjectId) -> &ArchivedObject<D> {
        &self.named_object(obj_id).object
    }

    pub fn root_objects(&self) -> &[ArchivedNamedObject<D>] {
        &self.objects
            [ArchivedObjectId::into_usize_range(&archived_range_to_range(&self.root_objects))]
    }

    pub fn named_objects(&self) -> impl Iterator<Item = &ArchivedNamedObject<D>> {
        self.objects.iter()
    }

    pub fn objects(&self) -> impl Iterator<Item = &ArchivedObject<D>> {
        self.named_objects()
            .map(|named_object| &named_object.object)
    }

    pub fn filter_objects<'a, O: IsArchivedObject<D> + 'a>(
        &'a self,
    ) -> impl Iterator<Item = (ArchivedObjectId, &'a O)> + 'a {
        self.objects()
            .enumerate()
            .filter_map(|(obj_id, obj)| Some((ArchivedObjectId::from_usize(obj_id), obj.as_()?)))
    }

    pub fn filter_objects_with<'a, O: IsArchivedObject<D> + 'a>(
        &'a self,
        f: impl 'a + Fn(&O) -> bool,
    ) -> impl Iterator<Item = (ArchivedObjectId, &'a O)> + 'a {
        self.filter_objects().filter(move |(_, obj)| (f)(obj))
    }

    pub fn lookup_object<'a, O: IsArchivedObject<D> + 'a>(
        &'a self,
        obj_id: ArchivedObjectId,
    ) -> Result<&'a O, TryFromObjectError> {
        self.object(obj_id).try_as()
    }
}

fn archived_range_to_range<T: Copy>(archived_range: &ArchivedRange<T>) -> Range<T> {
    archived_range.start..archived_range.end
}
