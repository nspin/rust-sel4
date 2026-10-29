//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use proc_macro2::{Spacing, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Token,
    parse::{ParseStream, Parser},
};

use super::{Condition, MacroImpls};

impl MacroImpls<'_> {
    pub fn cfg_select_impl(&self, toks: TokenStream) -> TokenStream {
        let input = match parse_cfg_select_input.parse2(toks) {
            Ok(parsed) => parsed,
            Err(err) => {
                return err.to_compile_error();
            }
        };
        for arm in input.arms.iter() {
            match &arm.condition {
                Some(condition) => match self.eval(condition) {
                    Ok(pass) => {
                        if pass {
                            return arm.body.clone();
                        }
                    }
                    Err(err) => {
                        return err.render();
                    }
                },
                None => {
                    return arm.body.clone();
                }
            }
        }
        let message = "none of the predicates in this `cfg_select` evaluated to true";
        quote! {
            compile_error!(#message);
        }
    }
}

struct CfgSelectInput {
    arms: Vec<Arm>,
}

struct Arm {
    // `None` for `_`
    condition: Option<Condition>,
    body: TokenStream,
}

fn parse_cfg_select_input(input: ParseStream) -> syn::Result<CfgSelectInput> {
    let mut arms = vec![];
    while !input.is_empty() {
        arms.push(parse_arm(input)?);
    }
    Ok(CfgSelectInput { arms })
}

fn parse_arm(input: ParseStream) -> syn::Result<Arm> {
    let condition = if input.peek(Token![_]) {
        input.parse::<Token![_]>()?;
        None
    } else {
        Some(syn::parse2(parse_until_fat_arrow(input)?)?)
    };
    input.parse::<Token![=>]>()?;
    let body = if input.peek(syn::token::Brace) {
        let body;
        syn::braced!(body in input);
        let body = body.parse()?;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
        body
    } else {
        let expr = input.parse::<syn::Expr>()?;
        if !input.is_empty() {
            input.parse::<Token![,]>()?;
        }
        expr.into_token_stream()
    };
    Ok(Arm { condition, body })
}

fn parse_until_fat_arrow(input: ParseStream) -> syn::Result<TokenStream> {
    input.step(|cursor| {
        let mut toks = TokenStream::new();
        let mut rest = *cursor;
        loop {
            if let Some((punct, next)) = rest.punct()
                && punct.as_char() == '='
                && punct.spacing() == Spacing::Joint
                && let Some((punct_, _)) = next.punct()
                && punct_.as_char() == '>'
            {
                return Ok((toks, rest));
            }
            match rest.token_tree() {
                Some((tt, next)) => {
                    toks.extend([tt]);
                    rest = next;
                }
                None => return Err(cursor.error("expected `=>`")),
            }
        }
    })
}
