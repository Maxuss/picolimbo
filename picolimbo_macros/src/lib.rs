use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput, GenericParam, Generics,
};

#[proc_macro_derive(Encodeable, attributes(varint))]
pub fn derive_encodeable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let method_body = generate_method_body(&input.data);

    let expanded = quote! {
        impl #impl_generics picolimbo_proto::Encodeable for #name #ty_generics #where_clause {
            fn encode(&self, buf: &mut picolimbo_proto::BytesMut) -> picolimbo_proto::Result<()> {
                #method_body
                Ok(())
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(parse_quote!(picolimbo_proto::Encodeable));
        }
    }
    generics
}

fn generate_method_body(data: &Data) -> TokenStream {
    let qt = match data {
        Data::Struct(ref dstruct) => match dstruct.fields {
            syn::Fields::Named(ref named) => {
                let recurse = named.named.iter().map(|f| {
                    let name = &f.ident;
                    let attrs = &f.attrs;
                    let self_value = if !attrs.is_empty()
                        && attrs.iter().any(|attr| attr.meta.path().is_ident("varint"))
                    {
                        quote_spanned! { f.span() => picolimbo_proto::Varint::from(self.#name)}
                    } else {
                        quote_spanned! { f.span() => self.#name}
                    };
                    quote_spanned! { f.span() =>
                        #self_value.encode(buf)?;
                    }
                });
                quote! {
                    #(#recurse)*
                }
            }
            syn::Fields::Unnamed(ref unnamed) => {
                let recurse = unnamed.unnamed.iter().map(|f| {
                    let name = &f.ident;
                    let attrs = &f.attrs;
                    let self_value = if !attrs.is_empty()
                        && attrs.iter().any(|attr| attr.meta.path().is_ident("varint"))
                    {
                        quote_spanned! { f.span() => picolimbo_proto::Varint::from(self.#name)}
                    } else {
                        quote_spanned! { f.span() => self.#name}
                    };
                    quote_spanned! { f.span() =>
                        #self_value.encode(buf)?;
                    }
                });
                quote! {
                    #(#recurse)*
                }
            }
            syn::Fields::Unit => {
                quote!()
            }
        },
        Data::Enum(_) => panic!("Enums are not supported for Encodeable"),
        Data::Union(_) => panic!("Unions are not supported for Encodeable"),
    };
    qt
}
