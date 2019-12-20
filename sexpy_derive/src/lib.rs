extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Sexpy)]
#[proc_macro_error]
pub fn sexpy_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a represntation of Rust code as a syntax tree
    // that we can manipulate
    let input = parse_macro_input!(input as DeriveInput);

    // Build the trait implementation
    impl_sexpy(&input).into()
}

fn field_parser(fields: &syn::Fields) -> Vec<TokenStream> {
    let field_iter = match fields {
        syn::Fields::Unnamed(fields) => fields.unnamed.iter(),
        syn::Fields::Named(fields) => fields.named.iter(),
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
            let ident = &segs.ident;
            if let syn::PathArguments::AngleBracketed(args) = &segs.arguments {
                quote! {
                    preceded(multispace0, #ident::#args::parser)
                }
            } else {
                quote! {
                    preceded(multispace1, #ident::parser)
                }
            }
        })
        .collect()
    // quote! {
    //     #( #toks ),*
    // }
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

fn variant_parser(id: &syn::Ident, var: &syn::Variant) -> TokenStream {
    let name = &var.ident;
    let head_name = &name.to_string().to_lowercase();
    let fields = field_parser(&var.fields);
    let args = field_arguments(&var.fields);
    if var.fields.len() == 1 {
        quote! {
            |i: &'a str| {
                let (next, #(#args)*) = head(
                    #head_name,
                    #( #fields )*
                )(i)?;
                Ok((next, #id::#name(#(#args)*)))
            }
        }
    } else {
        quote! {
            |i: &'a str| {
                let (next, (#(#args),*)) = head(
                    #head_name,
                    tuple((#( #fields ),*)),
                )(i)?;
                Ok((next, #id::#name(#(#args),*)))
            }
        }
    }
}

fn impl_sexpy(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let parser: TokenStream = match &ast.data {
        syn::Data::Enum(data) => {
            let ps: Vec<TokenStream> = data
                .variants
                .iter()
                .map(|var| variant_parser(name, var))
                .collect();
            if ps.len() == 1 {
                quote! {
                    #(#ps)*(input)
                }
            } else {
                quote! {
                    alt((#(#ps),*))(input)
                }
            }
        }
        syn::Data::Struct(data) => {
            let fields = field_parser(&data.fields);
            let head_name = &name.to_string().to_lowercase();
            let args = field_arguments(&data.fields);
            quote! {
                let (next, (#(#args),*)) = head(
                    #head_name,
                    tuple((#(preceded(multispace1, s_exp(#fields))),*)))(input)?;
                Ok((next, #name {
                    #(#args),*
                }))
            }
        }
        _ => abort_call_site!("not implemented for structs yet"),
    };
    quote! {
        impl Sexpy for #name {
            fn parser<'a>(input: &'a str) -> IResult<&'a str, Self, VerboseError<&'a str>>
            where
                Self: Sized {
                #parser
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
