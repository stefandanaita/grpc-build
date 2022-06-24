//! proc macro to generate an associate function for all pb message types
//! returning the full name of the message including the package namespace.

use proc_macro::TokenStream;
use syn::{spanned::Spanned, DeriveInput};

#[proc_macro_derive(FullyQualifiedName, attributes(name))]
pub fn fully_qualified_name(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast = syn::parse_macro_input!(input as DeriveInput);

    match impl_fully_qualified_name(&ast) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn impl_fully_qualified_name(ast: &syn::DeriveInput) -> syn::Result<TokenStream> {
    // We only annotate structs
    match &ast.data {
        syn::Data::Struct(_) => (),
        syn::Data::Enum(_) => return Ok(Default::default()),
        syn::Data::Union(_) => return Ok(Default::default()),
    };

    // search for #[name]
    let mut name_attrs: Vec<_> = ast
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("name"))
        .collect();

    if name_attrs.is_empty() {
        return Err(syn::Error::new(ast.span(), "missing #[name] attribute"));
    }

    // Let's assume we only have one annotation
    let attr = name_attrs.remove(0);
    let meta = attr.parse_meta()?;

    // #[name = "pbname"] should map to a NameValue
    //   path    Lit
    let message_name = if let syn::Meta::NameValue(value) = meta {
        if let syn::Lit::Str(name) = value.lit {
            name
        } else {
            return Err(syn::Error::new(
                value.lit.span(),
                "message name MUST be a string",
            ));
        }
    } else {
        return Err(syn::Error::new(meta.span(), "missing #[name] attribute"));
    };

    let name = &ast.ident;

    Ok(quote::quote! {
        impl #name {
            pub fn full_proto_name() -> &'static str {
                #message_name
            }
        }
    }
    .into())
}
