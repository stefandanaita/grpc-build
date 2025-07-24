//! proc macro to generate an associate function for all pb message types
//! returning the full name of the message including the package namespace.

use proc_macro::TokenStream;
use syn::{spanned::Spanned, DeriveInput, Expr, Lit, MetaNameValue};

#[proc_macro_derive(NamedMessage, attributes(name))]
pub fn fully_qualified_name(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);

    match impl_fully_qualified_name(&ast) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

fn impl_fully_qualified_name(ast: &syn::DeriveInput) -> syn::Result<TokenStream> {
    // We only annotate structs
    match &ast.data {
        syn::Data::Struct(_) => (),
        syn::Data::Enum(_) | syn::Data::Union(_) => return Ok(TokenStream::default()),
    };

    // search for #[name]
    let mut name_attrs = ast.attrs.iter().filter(|attr| attr.path().is_ident("name"));

    // Let's assume we only have one annotation
    let meta = match name_attrs.next() {
        Some(attr) => &attr.meta,
        None => return Err(syn::Error::new(ast.span(), "missing #[name] attribute")),
    };

    // #[name = "pbname"] should map to a NameValue
    //   path    Lit
    let message_name = match meta {
        syn::Meta::NameValue(MetaNameValue {
            value:
                Expr::Lit(syn::ExprLit {
                    lit: Lit::Str(name),
                    ..
                }),
            ..
        }) => name,
        syn::Meta::NameValue(MetaNameValue { value, .. }) => {
            return Err(syn::Error::new(
                value.span(),
                "message name MUST be a string",
            ))
        }
        meta => return Err(syn::Error::new(meta.span(), "missing #[name] attribute")),
    };

    let name = &ast.ident;

    Ok(quote::quote! {
        impl ::grpc_build_core::NamedMessage for #name {
            const NAME: &'static ::core::primitive::str = #message_name;
        }
    }
    .into())
}
