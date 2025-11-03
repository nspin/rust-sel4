//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use proc_macro::TokenStream;
use quote::{format_ident, quote};

#[proc_macro_derive(IsCap)]
pub fn derive_cap(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    derive_cap_impl(&ast)
}

fn derive_cap_impl(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let archived_name = format_ident!("Archived{}", name);
    quote! {
        impl<'b> TryFrom<&'b Cap> for &'b #name {
            type Error = TryFromCapError;
            fn try_from(cap: &'b Cap) -> Result<Self, Self::Error> {
                match cap {
                    Cap::#name(cap) => Ok(&cap),
                    _ => Err(TryFromCapError),
                }
            }
        }

        impl Into<Cap> for #name {
            fn into(self) -> Cap {
                Cap::#name(self)
            }
        }

        impl<'b> TryFrom<&'b ArchivedCap> for &'b #archived_name {
            type Error = TryFromCapError;
            fn try_from(cap: &'b ArchivedCap) -> Result<Self, Self::Error> {
                match cap {
                    ArchivedCap::#name(cap) => Ok(&cap),
                    _ => Err(TryFromCapError),
                }
            }
        }

        impl Into<ArchivedCap> for #archived_name {
            fn into(self) -> ArchivedCap {
                ArchivedCap::#name(self)
            }
        }
    }
    .into()
}

#[proc_macro_derive(IsObject)]
pub fn derive_object(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    derive_object_impl(&ast)
}

fn derive_object_impl(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let archived_name = format_ident!("Archived{}", name);
    let generics = &ast.generics;
    quote! {
        impl<'b, D, M> TryFrom<&'b Object<D, M>> for &'b #name #generics {
            type Error = TryFromObjectError;
            fn try_from(obj: &'b Object<D, M>) -> Result<Self, Self::Error> {
                match obj {
                    Object::#name(cap) => Ok(&cap),
                    _ => Err(TryFromObjectError),
                }
            }
        }

        impl<D, M> Into<Object<D, M>> for #name #generics {
            fn into(self) -> Object<D, M> {
                Object::#name(self)
            }
        }

        impl<'b, D: Archive, M: Archive> TryFrom<&'b ArchivedObject<D, M>> for &'b #archived_name #generics {
            type Error = TryFromObjectError;
            fn try_from(obj: &'b ArchivedObject<D, M>) -> Result<Self, Self::Error> {
                match obj {
                    ArchivedObject::#name(cap) => Ok(&cap),
                    _ => Err(TryFromObjectError),
                }
            }
        }

        impl<D: Archive, M: Archive> Into<ArchivedObject<D, M>> for #archived_name #generics {
            fn into(self) -> ArchivedObject<D, M> {
                ArchivedObject::#name(self)
            }
        }
    }
    .into()
}

#[proc_macro_derive(IsObjectWithCapTable)]
pub fn derive_object_with_cap_table(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    derive_object_with_cap_table_impl(&ast)
}

fn derive_object_with_cap_table_impl(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let archived_name = format_ident!("Archived{}", name);
    let generics = &ast.generics;
    quote! {
        impl #generics HasCapTable for #name #generics {
            fn slots(&self) -> &[CapTableEntry] {
                &*self.slots
            }
        }

        impl #generics HasArchivedCapTable for #archived_name #generics {
            fn slots(&self) -> &[ArchivedCapTableEntry] {
                &*self.slots
            }
        }
    }
    .into()
}
