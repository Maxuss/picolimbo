use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_quote, spanned::Spanned, Data, DeriveInput, Index};

use crate::common::{add_trait_bounds, parse_pfx_type};

pub fn expand_enc(input: DeriveInput) -> syn::Result<proc_macro::TokenStream> {
    let name = input.ident;

    let generics = add_trait_bounds(input.generics, parse_quote!(picolimbo_proto::Encodeable));
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let method_body = generate_method_body(&input.data)?;
    let size_body = generate_size_body(&input.data)?;

    let expanded = quote! {
        impl #impl_generics picolimbo_proto::Encodeable for #name #ty_generics #where_clause {
            fn encode(&self, buf: &mut picolimbo_proto::BytesMut) -> picolimbo_proto::Result<()> {
                #method_body
                Ok(())
            }

            fn predict_size(&self) -> usize {
                #size_body
            }
        }
    };

    Ok(proc_macro::TokenStream::from(expanded))
}

fn generate_size_body(data: &Data) -> syn::Result<TokenStream> {
    let qt: Result<TokenStream, syn::Error> = match data {
        Data::Struct(ref dstruct) => match dstruct.fields {
            syn::Fields::Named(ref named) => {
                let recurse = named.named.iter().map(|f| {
                    let name = &f.ident;
                    let attrs = &f.attrs;
                    
                    if attrs.iter().any(|attr| attr.meta.path().is_ident("varint")) {
                        quote_spanned! { f.span() => picolimbo_proto::Varint::size_of(self.#name as i32)}
                    } else if attrs.iter().any(|attr| attr.meta.path().is_ident("json")) {
                        quote_spanned! { f.span() => 0 } // we can not predict the size of a json structure, since serialization is expensive
                    } else if attrs.iter().any(|attr| attr.meta.path().is_ident("unprefixed")) {
                        quote_spanned! { f.span() => self.#name.len() }
                    } else if attrs.iter().any(|attr| attr.meta.path().is_ident("prefixed")) {
                        let parsed = parse_pfx_type(attrs);
                        quote_spanned! { f.span() => <#parsed>::pfx_size(self.#name.len()) + self.#name.iter().map(|each| each.predict_size()).sum::<usize>() }
                    } else {
                        quote_spanned! { f.span() => self.#name.predict_size() }
                    }
                });
                Ok(quote! {
                    #[allow(unused_imports)]
                    use picolimbo_proto::ArrayPrefix;

                    0 #(+ #recurse)*
                })
            }
            syn::Fields::Unnamed(ref unnamed) => {
                let mut idx = 0;
                let recurse = unnamed.unnamed.iter().map(|f| {
                    let name = Index::from(idx);
                    idx += 1;
                    let attrs = &f.attrs;
                    if attrs.iter().any(|attr| attr.meta.path().is_ident("varint"))
                    {
                        quote_spanned! { f.span() => picolimbo_proto::Varint::size_of(self.#name as i32)}
                    } else if attrs.iter().any(|attr| attr.meta.path().is_ident("json")) {
                        quote_spanned! { f.span() => 0 } // we can not predict the size of a json structure, since serialization is expensive
                    } else if attrs.iter().any(|attr| attr.meta.path().is_ident("unprefixed")) {
                        quote_spanned! { f.span() => self.#name.len() }
                    } else if attrs.iter().any(|attr| attr.meta.path().is_ident("prefixed")) {
                        let parsed = parse_pfx_type(attrs);
                        quote_spanned! { f.span() => <#parsed>::pfx_size(self.#name.len()) + self.#name.iter().map(|each| each.predict_size()).sum::<usize>() }
                    } else {
                        quote_spanned! { f.span() => self.#name.predict_size() }
                    }
                });
                Ok(quote! {
                    #[allow(unused_imports)]
                    use picolimbo_proto::ArrayPrefix;

                    0 #(+ #recurse)*
                })
            }
            syn::Fields::Unit => Ok(quote!(0)),
        },
        Data::Enum(dt) => Err(syn::Error::new_spanned(
            dt.enum_token,
            "Enums are not supported for Encodeable",
        )),
        Data::Union(dt) => Err(syn::Error::new_spanned(
            dt.union_token,
            "Unions are not supported for Encodeable",
        )),
    };
    qt
}

fn generate_method_body(data: &Data) -> syn::Result<TokenStream> {
    let qt: Result<TokenStream, syn::Error> = match data {
        Data::Struct(ref dstruct) => match dstruct.fields {
            syn::Fields::Named(ref named) => {
                let recurse = named.named.iter().map(|f| {
                    let name = &f.ident;
                    let attrs = &f.attrs;
                    let self_value = if attrs.iter().any(|attr| attr.meta.path().is_ident("varint"))
                    {
                        quote_spanned! { f.span() => picolimbo_proto::Varint::from(self.#name)}
                    } else if attrs.iter().any(|attr| attr.meta.path().is_ident("json")) {
                        quote_spanned! { f.span() => picolimbo_proto::JsonOut::from(&self.#name)}
                    } else if attrs.iter().any(|attr| attr.meta.path().is_ident("unprefixed")) {
                        quote_spanned! { f.span() => picolimbo_proto::UnprefixedByteArray(std::borrow::Cow::Borrowed(&self.#name)) }
                    } else if attrs.iter().any(|attr| attr.meta.path().is_ident("prefixed")) {
                        let parsed = parse_pfx_type(attrs);
                        quote_spanned! { f.span() => <#parsed>::array(std::borrow::Cow::Borrowed(&self.#name)) }
                    } else {
                        quote_spanned! { f.span() => self.#name}
                    };
                    quote_spanned! { f.span() =>
                        #self_value.encode(buf)?;
                    }
                });
                Ok(quote! {
                    #[allow(unused_imports)]
                    use picolimbo_proto::ArrayPrefix;

                    #(#recurse)*
                })
            }
            syn::Fields::Unnamed(ref unnamed) => {
                let mut idx = 0;
                let recurse = unnamed.unnamed.iter().map(|f| {
                    let name = Index::from(idx);
                    idx += 1;
                    let attrs = &f.attrs;
                    let self_value = if attrs.iter().any(|attr| attr.meta.path().is_ident("varint"))
                    {
                        quote_spanned! { f.span() => picolimbo_proto::Varint::from(self.#name)}
                    } else if attrs.iter().any(|attr| attr.meta.path().is_ident("json")) {
                        quote_spanned! { f.span() => picolimbo_proto::JsonOut::from(&self.#name)}
                    } else if attrs.iter().any(|attr| attr.meta.path().is_ident("unprefixed")) {
                        quote_spanned! { f.span() => picolimbo_proto::UnprefixedByteArray(std::borrow::Cow::Borrowed(&self.#name)) }
                    } else if attrs.iter().any(|attr| attr.meta.path().is_ident("prefixed")) {
                        let parsed = parse_pfx_type(attrs);
                        quote_spanned! { f.span() => <#parsed>::array(std::borrow::Cow::Borrowed(&self.#name)) }
                    } else {
                        quote_spanned! { f.span() => self.#name}
                    };
                    quote_spanned! { f.span() =>
                        #self_value.encode(buf)?;
                    }
                });
                Ok(quote! {
                    #[allow(unused_imports)]
                    use picolimbo_proto::ArrayPrefix;

                    #(#recurse)*
                })
            }
            syn::Fields::Unit => Ok(quote!()),
        },
        Data::Enum(dt) => Err(syn::Error::new_spanned(
            dt.enum_token,
            "Enums are not supported for Encodeable",
        )),
        Data::Union(dt) => Err(syn::Error::new_spanned(
            dt.union_token,
            "Unions are not supported for Encodeable",
        )),
    };
    qt
}
