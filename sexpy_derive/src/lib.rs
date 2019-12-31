mod attrs;

extern crate proc_macro;

use attrs::{FieldAttrs, SexpyAttr, TyAttrs};
use proc_macro2::{Span, TokenStream};
use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::quote;
use syn::{
    parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Fields, Ident,
    Variant,
};

#[proc_macro_derive(Sexpy, attributes(sexpy))]
#[proc_macro_error]
pub fn sexpy_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a represntation of Rust code as a syntax tree
    // that we can manipulate
    let input = parse_macro_input!(input as DeriveInput);

    // Build the trait implementation
    impl_sexpy(&input).into()
}

/// Processes the top level `DeriveInput`
fn impl_sexpy(ast: &DeriveInput) -> TokenStream {
    // name of the Struct or Enum
    let name = &ast.ident;

    // parse type level attributes
    let mut attrs = TyAttrs::from_attributes(&ast.attrs);

    // default head is `name`
    if attrs.head.is_none() {
        attrs.head = Some(name.to_string().to_lowercase())
    };

    // check what type of thing we have and call the corresponding
    // parser
    let parser: TokenStream = match &ast.data {
        Data::Enum(data) => enum_parser(&name, data, &attrs),
        Data::Struct(data) => struct_parser(&name, data, &mut attrs),
        _ => abort_call_site!("Only works on structs or enums"),
    };

    // construct Sexpy impl
    quote! {
        impl Sexpy for #name {
            fn sexp_parse<'a>(input: &'a str) ->
                IResult<&'a str, Self, VerboseError<&'a str>>
            where
                Self: Sized {
                #parser
            }
        }
    }
}

/// Generates the parser for `enum` types
fn enum_parser(
    parse_name: &Ident,
    data: &DataEnum,
    attrs: &TyAttrs,
) -> TokenStream {
    // abort if there are no variants
    if data.variants.len() == 0 {
        abort_call_site!("Can not construct enum with no cases.")
    }

    // generate a parser for each variant
    let parsers: Vec<TokenStream> = data
        .variants
        .iter()
        .map(|var| {
            let mut attrs = FieldAttrs::from_attributes(&var.attrs);
            variant_parser(parse_name, var, &mut attrs)
        })
        .collect();

    // we can't use `alt` if there is only one parser
    let parser = if parsers.len() == 1 {
        quote! {
            #( #parsers )*
        }
    } else {
        quote! {
            alt((#( #parsers ),*))
        }
    };

    // apply the attribute changes
    let ts = attrs.apply(parser);

    // construct final parser by applying the input
    quote! {
        #ts(input)
    }
}

/// Generates the parser for `struct` types
fn struct_parser(
    struct_name: &Ident,
    data: &DataStruct,
    attrs: &mut TyAttrs,
) -> TokenStream {
    // generate a parser for each field
    let fields = field_parser(&data.fields);

    // get the identifiers from the fields
    let idents = field_idents(&data.fields);
    let idents_str: Vec<String> =
        idents.iter().map(|x| x.to_string()).collect();
    let bindings = field_binder_syn(&idents);

    // turn the field parsers into a single tokenstream
    let parser = if data.fields.len() == 0 {
        quote! {
            multispace0
        }
    } else if data.fields.len() <= 1 {
        quote! {
            #(context(#idents_str, preceded(multispace1, #fields)))*
        }
    } else {
        quote! {
            tuple((
                #(context(#idents_str, preceded(multispace1, #fields))),*
            ))
        }
    };

    // apply the syntax changes from the attributes and construct
    // final syntax
    let ts = attrs.apply(parser);
    quote! {
        let (next, #bindings) = #ts(input)?;
        Ok((next, #struct_name { #(#idents),* }))
    }
}

/// Generates a vec of parsers that parse each field
/// in an enum or struct.
fn field_parser(fields: &Fields) -> Vec<TokenStream> {
    let field_iter = match fields {
        Fields::Unnamed(fields) => fields.unnamed.iter(),
        Fields::Named(fields) => fields.named.iter(),
        Fields::Unit => return vec![],
    };
    field_iter
        .map(|f| {
            let ty = &f.ty;
            quote! {
                <#ty>::sexp_parse
            }
        })
        .collect()
}

/// Generates a Vec of identifiers from field names
fn field_idents(fields: &Fields) -> Vec<Ident> {
    match fields {
        Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(idx, _)| {
                Ident::new(&format!("a_{}", idx), Span::call_site())
            })
            .collect(),
        Fields::Named(fields) => fields
            .named
            .iter()
            .map(|f| match &f.ident {
                Some(id) => id.clone(),
                None => abort_call_site!("Expected named field"),
            })
            .collect(),
        Fields::Unit => vec![],
    }
}

/// Generates a parser for a single variant in an enum type.
fn variant_parser(
    id: &Ident,
    var: &Variant,
    attrs: &mut FieldAttrs,
) -> TokenStream {
    let name = &var.ident;
    let fld_par = field_parser(&var.fields);
    let idents = field_idents(&var.fields);
    let binders = field_binder_syn(&idents);

    let field_syn = if var.fields.len() == 0 {
        quote! { multispace0 }
    } else if var.fields.len() == 1 {
        quote! {
            #( preceded(multispace1, #fld_par) )*
        }
    } else {
        quote! {
            tuple((#( preceded(multispace1, #fld_par) ),*))
        }
    };

    // check if the enum takes arguments
    let enum_constr = if var.fields.len() == 0 {
        quote! { #id::#name }
    } else {
        quote! { #id::#name(#(#idents),*) }
    };

    // apply attribute syntax changes and construct final parser
    let ts = attrs.apply(field_syn);
    quote! {
        |i: &'a str| {
            let (next, #binders) = #ts(i)?;
            Ok((next, #enum_constr))
        }
    }
}

/// Helper function to generate the syntax for binding and deconstructing
/// identifers that we get from calling parsers
fn field_binder_syn(idents: &Vec<Ident>) -> TokenStream {
    if idents.len() == 0 {
        quote! { _ }
    } else if idents.len() == 1 {
        quote! { #(#idents),* }
    } else {
        quote! { (#(#idents),*) }
    }
}
