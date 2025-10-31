//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::string::String;
use core::str;

use rkyv::option::ArchivedOption;
use rkyv::string::ArchivedString;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{NamedObject, Object};

pub trait ObjectName {
    fn object_name(&self) -> Option<&str>;
}

impl ObjectName for &str {
    fn object_name(&self) -> Option<&str> {
        Some(self)
    }
}

impl ObjectName for String {
    fn object_name(&self) -> Option<&str> {
        Some(self)
    }
}

impl ObjectName for ArchivedString {
    fn object_name(&self) -> Option<&str> {
        Some(self)
    }
}

impl ObjectName for ArchivedUnnamed {
    fn object_name(&self) -> Option<&str> {
        None
    }
}

impl<T: ObjectName> ObjectName for Option<T> {
    fn object_name(&self) -> Option<&str> {
        self.as_ref().and_then(ObjectName::object_name)
    }
}

impl<T: ObjectName> ObjectName for ArchivedOption<T> {
    fn object_name(&self) -> Option<&str> {
        self.as_ref().and_then(ObjectName::object_name)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct Unnamed;

impl ObjectName for Unnamed {
    fn object_name(&self) -> Option<&str> {
        None
    }
}

// // //

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ObjectNamesLevel {
    All,
    JustTcbs,
    None,
}

impl ObjectNamesLevel {
    pub fn apply<'a, N, D, M>(&self, named_obj: &'a NamedObject<N, D, M>) -> Option<&'a N> {
        match self {
            Self::All => Some(&named_obj.name),
            Self::JustTcbs => match &named_obj.object {
                Object::Tcb(_) => Some(&named_obj.name),
                _ => None,
            },
            Self::None => None,
        }
    }
}
