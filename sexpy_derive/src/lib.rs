mod attrs;

extern crate proc_macro;

use attrs::Attrs;
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

    let attrs = Attrs::from_attributes(&ast.attrs);

    let parser: TokenStream = match &ast.data {
        Data::Enum(data) => enum_parser(&name, data, &attrs),
        Data::Struct(data) => struct_parser(&name, data, &attrs),
        _ => abort_call_site!("Only works on structs or enums"),
    };

    // construct Sexpy impl
    quote! {
        impl Sexpy for #name {
            fn sexp_parse<'a>(input: &'a str) -> IResult<&'a str, Self, VerboseError<&'a str>>
            where
                Self: Sized {
                #parser
            }
        }
    }
}

fn enum_parser(
    parse_name: &Ident,
    data: &DataEnum,
    attrs: &Attrs,
) -> TokenStream {
    let parsers: Vec<TokenStream> = data
        .variants
        .iter()
        .map(|var| {
            let attrs = Attrs::from_attributes(&var.attrs);
            variant_parser(parse_name, var, &attrs)
        })
        .collect();
    let parser = if parsers.len() == 1 {
        quote! {
            #( #parsers )*
        }
    } else {
        quote! {
            alt((#( #parsers ),*))
        }
    };
    match &attrs.name {
        Some(name) => quote! {
            head(#name, #parser)(input)
        },
        None => quote! {
            #parser(input)
        },
    }
}

fn struct_parser(
    parse_name: &Ident,
    data: &DataStruct,
    attrs: &Attrs,
) -> TokenStream {
    let fields = field_parser(&data.fields);
    let head_name = match &attrs.name {
        Some(s) => s.clone(),
        None => parse_name.to_string().to_lowercase(),
    };
    let args = field_arguments(&data.fields);
    let args_str: Vec<String> = args.iter().map(|x| x.to_string()).collect();
    quote! {
        let (next, (#(#args),*)) = head(
            #head_name,
            tuple((
                #(context(#args_str, preceded(multispace1, #fields))),*
            )))(input)?;
        Ok((next, #parse_name {
            #(#args),*
        }))
    }
}

fn field_parser(fields: &Fields) -> Vec<TokenStream> {
    let field_iter = match fields {
        Fields::Unnamed(fields) => fields.unnamed.iter(),
        Fields::Named(fields) => fields.named.iter(),
        _ => abort_call_site!("fields"),
    };
    field_iter
        .map(|f| {
            let segs = match &f.ty {
                syn::Type::Path(syn::TypePath {
                    qself: None,
                    path:
                        syn::Path {
                            leading_colon: None,
                            segments,
                        },
                }) => segments.iter().last(),
                _ => None,
            }
            .unwrap();
            let type_name = &segs.ident;
            if let syn::PathArguments::AngleBracketed(args) = &segs.arguments {
                quote! {
                    #type_name::#args::sexp_parse
                }
            } else {
                quote! {
                    #type_name::sexp_parse
                }
            }
        })
        .collect()
}

fn field_arguments(fields: &syn::Fields) -> Vec<syn::Ident> {
    match fields {
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(idx, _)| {
                syn::Ident::new(&format!("a_{}", idx), Span::call_site())
            })
            .collect(),
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .map(|f| match &f.ident {
                Some(id) => id.clone(),
                None => abort_call_site!("Expected named field"),
            })
            .collect(),
        _ => abort_call_site!("Don't support this kind of field"),
    }
}

fn variant_parser(id: &Ident, var: &Variant, attrs: &Attrs) -> TokenStream {
    let name = &var.ident;
    let head_name = match &attrs.name {
        Some(name) => name.clone(),
        None => name.to_string().to_lowercase(),
    };
    let fields = field_parser(&var.fields);
    let args = field_arguments(&var.fields);
    let (arg_syn, field_syn) = if var.fields.len() == 1 {
        (
            quote! {
                #( #args )*
            },
            quote! {
                #( preceded(multispace1, #fields) )*
            },
        )
    } else {
        (
            quote! {
                (#( #args ),*)
            },
            quote! {
                tuple((#( preceded(multispace1, #fields) ),*))
            },
        )
    };
    quote! {
        |i: &'a str| {
            let (next, #arg_syn) = head(
                #head_name,
                #field_syn,
            )(i)?;
            Ok((next, #id::#name(#(#args),*)))
        }
    }
}
