use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Attribute, Error, Ident, LitStr, Token,
};

// ============ Trait Definitions ================ //

struct SexpyAttrSyn<EnumT> {
    attrs: Punctuated<EnumT, Token![,]>,
}

impl<EnumT: Parse> Parse for SexpyAttrSyn<EnumT> {
    fn parse(input: ParseStream) -> Result<Self> {
        // let _paren = parenthesized!(content in input);
        let attrs = input.parse_terminated(EnumT::parse)?;

        Ok(SexpyAttrSyn { attrs })
    }
}

pub trait SexpyAttr<EnumT: Parse> {
    fn empty() -> Self;
    fn add_enum(&mut self, enm: &EnumT);
    fn from_attributes(attributes: &[Attribute]) -> Self
    where
        Self: Sized,
    {
        let attr_enums: Vec<EnumT> = attributes
            .iter()
            .map(|attr| match attr.parse_args_with(SexpyAttrSyn::parse) {
                Ok(a_enum) => a_enum,
                Err(e) => abort!(e.span(), "Unknown field"),
            })
            .map(|attr_syn| attr_syn.attrs)
            .flatten()
            .collect();
        let mut res = Self::empty();
        for e in attr_enums {
            res.add_enum(&e);
        }
        res
    }
}

pub trait ApplyAttr {
    fn apply(&self, ts: TokenStream) -> TokenStream;
}

// =============== Type Level Attributes ================ //

pub struct TyAttrs {
    pub head: Option<String>,
    pub surround: bool,
}

#[derive(Debug)]
pub enum TyAttrEnum {
    Head(String, Span),
    Surround(bool, Span),
}

impl SexpyAttr<TyAttrEnum> for TyAttrs {
    fn empty() -> Self {
        TyAttrs {
            head: None,
            surround: true,
        }
    }

    fn add_enum(&mut self, e: &TyAttrEnum) {
        use TyAttrEnum::*;
        match e {
            Head(s, _) => self.head = Some(s.to_string()),
            Surround(b, _) => self.surround = *b,
        }
    }
}

impl Parse for TyAttrEnum {
    fn parse(input: ParseStream) -> Result<Self> {
        use TyAttrEnum::*;

        let field: Ident = input.parse()?;

        match field.to_string().as_ref() {
            "head" => {
                let _ = input.parse::<Token![=]>()?;
                let lit: LitStr = input.parse()?;
                let lit_val = lit.value();
                Ok(Head(lit_val, lit.span()))
            }
            "nosurround" => Ok(Surround(false, field.span())),
            _ => Err(Error::new(
                field.span(),
                format!("expected `name`, found {}", field),
            )),
        }
    }
}

impl ApplyAttr for TyAttrs {
    fn apply(&self, ts: TokenStream) -> TokenStream {
        let mut res = ts;
        if let Some(head) = &self.head {
            res = quote! { head(#head, #res) }
        };

        if self.surround {
            res = quote! { (|i: &'a str| surround(#res, i)) }
        }

        res
    }
}

// ============ field level attributes ================ //
pub struct FieldAttrs {
    pub head: Option<String>,
    pub surround: bool,
}

#[derive(Debug)]
pub enum FieldAttrEnum {
    Head(String, Span),
    Surround(bool, Span),
}

impl SexpyAttr<FieldAttrEnum> for FieldAttrs {
    fn empty() -> Self {
        FieldAttrs {
            head: None,
            surround: false,
        }
    }

    fn add_enum(&mut self, e: &FieldAttrEnum) {
        use FieldAttrEnum::*;
        match e {
            Head(s, _) => self.head = Some(s.to_string()),
            Surround(b, _) => self.surround = *b,
        }
    }
}

impl Parse for FieldAttrEnum {
    fn parse(input: ParseStream) -> Result<Self> {
        use FieldAttrEnum::*;
        let field: Ident = input.parse()?;

        match field.to_string().as_ref() {
            "head" => {
                let _ = input.parse::<Token![=]>()?;
                let lit: LitStr = input.parse()?;
                let lit_val = lit.value();
                Ok(Head(lit_val, lit.span()))
            }
            "surround" => Ok(Surround(true, field.span())),
            "nosurround" => Ok(Surround(false, field.span())),
            _ => Err(Error::new(
                field.span(),
                format!("expected `name`, found {}", field),
            )),
        }
    }
}

impl ApplyAttr for FieldAttrs {
    fn apply(&self, ts: TokenStream) -> TokenStream {
        let mut res = ts;
        if let Some(head) = &self.head {
            res = quote! { head(#head, #res) }
        };

        if self.surround {
            res = quote! { (|i: &'a str| surround(#res, i)) }
        }

        res
    }
}
