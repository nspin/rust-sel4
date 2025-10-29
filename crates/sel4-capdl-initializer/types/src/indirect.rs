//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::borrow::Borrow;
use core::fmt;
use core::ops::Deref;

#[cfg(feature = "indirect-owned")]
use core::marker::PhantomData;

#[cfg(all(feature = "indirect-owned", not(feature = "std")))]
// #[cfg(feature = "indirect-owned")]
use alloc::{boxed::Box, vec::Vec};

use cfg_if::cfg_if;

#[cfg(feature = "serde")]
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

#[cfg(not(any(feature = "indirect-borrowed", feature = "indirect-owned",)))]
compile_error!(r#"at least one of "indirect-borrowed" or "indirect-owned" must be enabled"#);

pub trait IndirectTarget {
    #[cfg(feature = "indirect-owned")]
    type Owned: Borrow<Self>;

    // May be unecessary, depending on whether postcard gives size hints
    #[cfg(feature = "indirect-owned")]
    fn shrink_to_fit(_owned: &mut Self::Owned) {}
}

cfg_if! {
    if #[cfg(feature = "indirect-owned")] {
        impl<T> IndirectTarget for T {
            type Owned = Box<T>;
        }

        impl<T> IndirectTarget for [T] {
            type Owned = Vec<T>;

            fn shrink_to_fit(owned: &mut Self::Owned) {
                owned.shrink_to_fit();
            }
        }
    } else {
        impl<T: ?Sized> IndirectCompatible for T {}
    }
}

// NOTE
// Box<[T]> vs Vec<T>: Box<[T]> has no wasted capacity, but is always constructed by first constructing a Vec<T>.
// Does this have a positive or negative impact on memory footprint?
// Minimization but possibly incurring fragmentation.

pub struct Indirect<'a, T: IndirectTarget + ?Sized>(IndirectImpl<'a, T>);

enum IndirectImpl<'a, T: IndirectTarget + ?Sized> {
    #[cfg(feature = "indirect-borrowed")]
    Borrowed { borrowed: &'a T },
    #[cfg(feature = "indirect-owned")]
    Owned {
        owned: T::Owned,
        phantom: PhantomData<&'a ()>,
    },
}

#[allow(clippy::needless_lifetimes)]
impl<'a, T: IndirectTarget + ?Sized> Indirect<'a, T> {
    #[cfg(feature = "indirect-borrowed")]
    pub const fn from_borrowed(borrowed: &'a T) -> Self {
        Self(IndirectImpl::Borrowed { borrowed })
    }

    #[cfg(feature = "indirect-owned")]
    pub const fn from_owned(owned: T::Owned) -> Self {
        Self(IndirectImpl::Owned {
            owned,
            phantom: PhantomData,
        })
    }

    fn inner(&self) -> &T {
        match self.0 {
            #[cfg(feature = "indirect-borrowed")]
            IndirectImpl::Borrowed { borrowed } => borrowed,
            #[cfg(feature = "indirect-owned")]
            IndirectImpl::Owned { ref owned, .. } => owned.borrow(),
        }
    }

    #[cfg(feature = "indirect-borrowed")]
    pub const fn const_inner(&self) -> &T {
        match self.0 {
            #[cfg(feature = "indirect-indirect")]
            IndirectImpl::Borrowed { borrowed } => borrowed,
            #[cfg(feature = "indirect-owned")]
            IndirectImpl::Owned { .. } => panic!(),
        }
    }
}

impl<T: IndirectTarget + ?Sized> Deref for Indirect<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner()
    }
}

impl<T: IndirectTarget + ?Sized> Borrow<T> for Indirect<'_, T> {
    fn borrow(&self) -> &T {
        self.inner()
    }
}

impl<T: Clone> Clone for Indirect<'_, T> {
    fn clone(&self) -> Self {
        Self(match self.0 {
            #[cfg(feature = "indirect-borrowed")]
            IndirectImpl::Borrowed { borrowed } => IndirectImpl::Borrowed { borrowed },
            #[cfg(feature = "indirect-owned")]
            IndirectImpl::Owned { ref owned, phantom } => IndirectImpl::Owned {
                owned: owned.clone(),
                phantom,
            },
        })
    }
}

impl<T: Clone> Clone for Indirect<'_, [T]> {
    fn clone(&self) -> Self {
        Self(match self.0 {
            #[cfg(feature = "indirect-borrowed")]
            IndirectImpl::Borrowed { borrowed } => IndirectImpl::Borrowed { borrowed },
            #[cfg(feature = "indirect-owned")]
            IndirectImpl::Owned { ref owned, phantom } => IndirectImpl::Owned {
                owned: owned.clone(),
                phantom,
            },
        })
    }
}

impl<T: IndirectTarget + fmt::Debug + ?Sized> fmt::Debug for Indirect<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner().fmt(f)
    }
}

impl<'b, T: IndirectTarget + ?Sized, U: IndirectTarget + ?Sized> PartialEq<Indirect<'b, U>>
    for Indirect<'_, T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &Indirect<'b, U>) -> bool {
        self.inner().eq(other.inner())
    }
}

impl<T: IndirectTarget + Eq + ?Sized> Eq for Indirect<'_, T> {}

#[cfg(feature = "indirect-owned")]
impl<T: IndirectTarget> FromIterator<T> for Indirect<'_, [T]> {
    fn from_iter<U>(iter: U) -> Self
    where
        U: IntoIterator<Item = T>,
    {
        Self::from_owned(iter.into_iter().collect())
    }
}

#[cfg(feature = "serde")]
impl<T: IndirectTarget + Serialize + ?Sized> Serialize for Indirect<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.inner().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: IndirectTarget + ?Sized> Deserialize<'de> for Indirect<'_, T>
where
    T::Owned: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Deserialize::deserialize(deserializer).map(|mut x| {
            T::shrink_to_fit(&mut x);
            Indirect::from_owned(x)
        })
    }
}
