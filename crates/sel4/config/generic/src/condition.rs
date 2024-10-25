use syn::parse::{Parse, ParseStream, Result as ParseResult};
use fallible_iterator::FallibleIterator;
use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::spanned::Spanned;

use sel4_config_generic_types::{Configuration, Value};

pub(crate) enum Condition {
    Key(syn::Ident),
    KeyValue(syn::Ident, syn::LitStr),
    Not(Box<Condition>),
    All(Vec<Condition>),
    Any(Vec<Condition>),
}

impl Condition {
    pub(crate) fn eval(&self, config: &Configuration) -> Result<bool, EvalError> {
        ConfigurationForEval(config).eval(self)
    }
}

impl Parse for Condition {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        todo!()
    }
}

pub(crate) struct EvalError {
    pub(crate) span: Span,
    pub(crate) message: String,
}

impl EvalError {
    pub(crate) fn new(span: Span, message: String) -> Self {
        Self { span, message }
    }

    pub(crate) fn render(&self) -> TokenStream {
        let message = &self.message;
        quote_spanned! {
            self.span => compile_error!(#message);
        }
    }
}

fn err<T, U: ToString>(node: impl Spanned, message: U) -> Result<T, EvalError> {
    Err(EvalError::new(node.span(), message.to_string()))
}

struct ConfigurationForEval<'a>(&'a Configuration);

impl<'a> ConfigurationForEval<'a> {
    fn lookup_key(&self, k: &syn::Ident) -> Result<&Value, EvalError> {
        self.0.get(&k.to_string()).ok_or_else(|| EvalError::new(k.span(), format!("unknown config key '{k}'")))
    }

    fn eval(&self, cond: &Condition) -> Result<bool, EvalError> {
        todo!()
        // match cond {
        // }
    }
}

// impl<'a> MacroImpls<'a> {
//     pub(crate) fn eval_condition(&self, node: &Condition) -> Result<bool, EvalError> {
//         Ok(match node {
//             syn::NestedMeta::Meta(node) => self.eval_meta(node)?,
//             syn::NestedMeta::Lit(node) => match node {
//                 syn::Lit::Bool(node) => node.value,
//                 _ => return err(node, "unexpected literal type"),
//             },
//         })
//     }

//     fn eval_meta(&self, node: &syn::Meta) -> Result<bool, EvalError> {
//         Ok(match node {
//             syn::Meta::Path(node) => {
//                 match self.lookup_path(node)? {
//                     Value::Bool(v) => *v,
//                     _ => return err(node, "config key does not correspond to a boolean"),
//                 }
//             }
//             syn::Meta::NameValue(node) => {
//                 match (&node.lit, self.lookup_path(&node.path)?) {
//                     (syn::Lit::Str(l), Value::String(v)) => &l.value() == v,
//                     (syn::Lit::Bool(l), Value::Bool(v)) => &l.value() == v,
//                     _ => return err(node, "the type of the value corresponding to config key does not match the type of the value to which it is being compared"),
//                 }
//             }
//             syn::Meta::List(node) => {
//                 match node.path.get_ident() {
//                     None => return err(&node.path, "unknown operation"),
//                     Some(ident) => match ident.to_string().as_str() {
//                         "not" => {
//                             if node.nested.len() != 1 {
//                                 return err(&node.nested, "expected 1 argument")
//                             }
//                             !self.eval_nested_meta(node.nested.first().unwrap())?
//                         }
//                         "any" => {
//                             fallible_iterator::convert(node.nested.iter().map(Ok)).any(|e| {
//                                 self.eval_nested_meta(e)
//                             })?
//                         }
//                         "all" => {
//                             fallible_iterator::convert(node.nested.iter().map(Ok)).all(|e| {
//                                 self.eval_nested_meta(e)
//                             })?
//                         }
//                         _ => {
//                             return err(&node.path, "unknown operation")
//                         }
//                     }
//                 }
//             }
//         })
//     }


// }