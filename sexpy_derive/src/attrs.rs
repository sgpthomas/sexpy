use proc_macro2::Span;
use proc_macro_error::abort;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Attribute, Error, Ident, LitStr, Token,
};

pub struct Attrs {
    pub name: Option<String>,
}

impl Attrs {
    fn empty() -> Self {
        Attrs { name: None }
    }

    fn from_enums(enums: &[AttrEnum]) -> Self {
        let mut res = Attrs::empty();
        for e in enums {
            match e {
                AttrEnum::Name(s, _) => res.name = Some(s.to_string()),
            }
        }
        res
    }

    pub fn from_attributes(attributes: &[Attribute]) -> Self {
        let attr_enums: Vec<AttrEnum> = attributes
            .iter()
            .map(
                |attr| match attr.parse_args_with(SexpyAttributesSyn::parse) {
                    Ok(a_enum) => a_enum,
                    Err(e) => abort!(e.span(), "Unknown field"),
                },
            )
            .map(|attr_syn| attr_syn.attrs)
            .flatten()
            .collect();
        Self::from_enums(&attr_enums)
    }
}

#[derive(Debug)]
enum AttrEnum {
    Name(String, Span),
}

impl Parse for AttrEnum {
    fn parse(input: ParseStream) -> Result<Self> {
        let field: Ident = input.parse()?;
        let _ = input.parse::<Token![=]>()?;
        let lit: LitStr = input.parse()?;
        let lit_val = lit.value();

        if field.to_string() == "name" {
            Ok(AttrEnum::Name(lit_val, lit.span()))
        } else {
            Err(Error::new(
                field.span(),
                format!("expected `name`, found {}", field),
            ))
        }
    }
}

struct SexpyAttributesSyn {
    attrs: Punctuated<AttrEnum, Token![,]>,
}

impl Parse for SexpyAttributesSyn {
    fn parse(input: ParseStream) -> Result<Self> {
        // let _paren = parenthesized!(content in input);
        let attrs = input.parse_terminated(AttrEnum::parse)?;

        Ok(SexpyAttributesSyn { attrs })
    }
}
