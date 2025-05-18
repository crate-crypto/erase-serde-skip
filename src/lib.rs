extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Meta, parse_macro_input};

/// Retain only the parts of `#[serde(...)]` that are **not**
/// `skip_serializing_if = ...`.
fn strip_field(field: &mut syn::Field) {
    field.attrs.retain_mut(|attr| {
        if !attr.path().is_ident("serde") {
            return true; // keep non-serde attrs
        }

        // Parse the #[serde(...)] list into a Vec<Meta>
        let metas: syn::punctuated::Punctuated<Meta, syn::token::Comma> = match attr
            .parse_args_with(
                syn::punctuated::Punctuated::<Meta, syn::token::Comma>::parse_terminated,
            ) {
            Ok(m) => m,
            Err(_) => return true, // malformed → leave it alone
        };

        // Drop every skip_serializing_if
        let kept = metas
            .into_iter()
            .filter(
                |m| !matches!(m, Meta::NameValue(nv) if nv.path.is_ident("skip_serializing_if")),
            )
            .collect::<syn::punctuated::Punctuated<Meta, syn::token::Comma>>();

        if kept.is_empty() {
            return false; // nothing left → drop the whole attribute
        }

        // Rewrite the tokens inside #[serde(...)]
        if let Meta::List(list) = &mut attr.meta {
            list.tokens = quote!(#kept);
        }

        true
    });
}

/// Apply `strip_field` to every field in `fields`, no matter the form.
fn process_fields(fields: &mut Fields) {
    for field in fields.iter_mut() {
        strip_field(field);
    }
}

/// Remove all `skip_serializing_if` clauses **everywhere** in a struct or enum.
#[proc_macro_attribute]
pub fn erase_skip_serializing_if(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as DeriveInput);

    match &mut input.data {
        Data::Struct(strukt) => process_fields(&mut strukt.fields),
        Data::Enum(enm) => {
            for variant in &mut enm.variants {
                process_fields(&mut variant.fields);
            }
        }
        Data::Union(_) => {} // unions: nothing to do
    }

    TokenStream::from(quote!(#input))
}
