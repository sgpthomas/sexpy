use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Attribute, Error, Ident, LitStr, Token,
};

// ============ Trait Definitions ================ //

/// Struct that represents the Sexpy attribute syntax.
/// The syntax is simply a comma separated list of syntax
/// defined by the parameter `EnumT`.
struct SexpyAttrSyn<EnumT> {
    attrs: Punctuated<EnumT, Token![,]>,
}

/// Generically implement parse for SexpyAttrSyn over
/// parsable types.
impl<EnumT: Parse> Parse for SexpyAttrSyn<EnumT> {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.parse_terminated(EnumT::parse)?;

        Ok(SexpyAttrSyn { attrs })
    }
}

/// A trait that handles generalizing definitions and syntax
/// made over single attributes to lists of attributes
pub trait SexpyAttr<EnumT: Parse> {
    /// The default constructor of Self. Represents
    /// the default settings of the attributes
    fn default() -> Self;

    /// Given a mutable reference to self, make the minimum
    /// changes to update `self` with `enm`.
    fn add_enum(&mut self, enm: &EnumT);

    /// Modify a token stream with the attributes in self
    fn apply(&self, ts: TokenStream) -> TokenStream;

    /// Generate `Self` from a slice of `syn::Atribute` syntax
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
        let mut res = Self::default();
        for e in attr_enums {
            res.add_enum(&e);
        }
        res
    }
}

// =============== Type Level Attributes ================ //
pub struct TyAttrs {
    pub nohead: bool,
    pub head: Option<String>,
    pub surround: bool,
}

#[derive(Debug)]
pub enum TyAttrEnum {
    NoHead(bool, Span),
    Head(String, Span),
    Surround(bool, Span),
}

impl SexpyAttr<TyAttrEnum> for TyAttrs {
    fn default() -> Self {
        TyAttrs {
            nohead: false,
            head: None,
            surround: true,
        }
    }

    fn apply(&self, ts: TokenStream) -> TokenStream {
        let mut res = ts;

        if !self.nohead {
            if let Some(head) = &self.head {
                res = quote! { (::sexpy::parsers::head(#head, #res)) }
            }
        }

        if self.surround {
            res = quote! { (|i: &'a str| ::sexpy::parsers::surround(#res, i)) }
        }

        res
    }

    fn add_enum(&mut self, e: &TyAttrEnum) {
        use TyAttrEnum::*;
        match e {
            NoHead(b, _) => self.nohead = *b,
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
            "nohead" => Ok(NoHead(true, field.span())),
            "nosurround" => Ok(Surround(false, field.span())),
            _ => Err(Error::new(
                field.span(),
                format!("expected `name`, found {}", field),
            )),
        }
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
    fn default() -> Self {
        FieldAttrs {
            head: None,
            surround: false,
        }
    }

    fn apply(&self, ts: TokenStream) -> TokenStream {
        let mut res = ts;
        if let Some(head) = &self.head {
            res = quote! {
                nom::sequence::preceded(
                    ::sexpy::parsers::wordbreak0,
                    ::sexpy::parsers::head(#head, #res))
            }
        };

        if self.surround {
            res = quote! { (|i: &'a str| ::sexpy::parsers::surround(#res, i)) }
        };

        res
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
            // "nosurround" => Ok(Surround(false, field.span())),
            _ => Err(Error::new(
                field.span(),
                format!("expected `name`, found {}", field),
            )),
        }
    }
}
