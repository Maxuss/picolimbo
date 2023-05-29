use syn::{parse_macro_input, DeriveInput};

pub(crate) mod common;

mod dec;
mod enc;

#[proc_macro_derive(Encodeable, attributes(varint))]
pub fn derive_encodeable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    enc::expand_enc(input).unwrap_or_else(|e| syn::Error::into_compile_error(e).into())
}

#[proc_macro_derive(Decodeable, attributes(varint, string))]
pub fn derive_decodeable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    dec::expand_dec(input).unwrap_or_else(|e| syn::Error::into_compile_error(e).into())
}
