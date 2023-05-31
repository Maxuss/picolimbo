use proc_macro2::Ident;
use quote::{quote, quote_spanned};
use syn::{parse_quote, spanned::Spanned, Data, DeriveInput};

use crate::common::{add_trait_bounds, parse_pfx_type};

pub fn expand_dec(input: DeriveInput) -> syn::Result<proc_macro::TokenStream> {
    let name = input.ident;

    let generics = add_trait_bounds(input.generics, parse_quote!(picolimbo_proto::Decodeable));
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let method_body = generate_method_body(&input.data)?;

    let expanded = quote! {
        impl #impl_generics picolimbo_proto::Decodeable for #name #ty_generics #where_clause {
            fn decode(read: &mut std::io::Cursor<&[u8]>, ver: picolimbo_proto::Protocol) -> picolimbo_proto::Result<Self> where Self: Sized {
                #method_body
            }
        }
    };

    Ok(proc_macro::TokenStream::from(expanded))
}

fn generate_method_body(data: &syn::Data) -> syn::Result<proc_macro2::TokenStream> {
    let qt = match data {
        Data::Struct(ref dstruct) => match dstruct.fields {
            syn::Fields::Named(ref named) => {
                let recurse = named.named.iter().map(|f| {
                    let name = &f.ident;
                    let ty = &f.ty;
                    let attrs = &f.attrs;
                    
                    if attrs.iter().any(|attr| attr.meta.path().is_ident("varint"))
                    {
                        quote_spanned! { f.span() => let #name = picolimbo_proto::Varint::decode(read, ver)?.0 as #ty; }
                    } else if attrs.iter().any(|attr| attr.meta.path().is_ident("unprefixed")) {
                        quote_spanned! { f.span() => let #name = (*picolimbo_proto::UnprefixedByteArray::decode(read, ver)?.0).to_owned() }
                    } else if attrs.iter().any(|attr| attr.meta.path().is_ident("prefixed")) {
                        let parsed = parse_pfx_type(attrs);
                        let generics = &f.ty;
                        quote_spanned! { f.span() => let #name: #generics = (*<#parsed>::decoding(read, ver)?.0).to_owned(); }
                    } else {
                        quote_spanned! { f.span() => let #name = <#ty>::decode(read, ver)?; }
                    }
                });
                let self_builder = named.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned! { f.span() => #name, }
                });
                Ok(quote! {
                    #[allow(unused_imports)]
                    use picolimbo_proto::ArrayPrefix;
                    #(#recurse)*;
                    Ok(Self {
                        #(#self_builder)*
                    })
                })
            }
            syn::Fields::Unnamed(ref unnamed) => {
                let mut idx = 0;
                let recurse = unnamed.unnamed.iter().map(|f| {
                    let field_name = Ident::new(&format!("field_{idx}"), f.span());
                    idx += 1;
                    let ty = &f.ty;
                    let attrs = &f.attrs;
                    
                    if attrs.iter().any(|attr| attr.meta.path().is_ident("varint"))
                    {
                        quote_spanned! { f.span() => let #field_name = picolimbo_proto::Varint::decode(read, ver)?.0 as #ty;}
                    } else if attrs.iter().any(|attr| attr.meta.path().is_ident("unprefixed")) {
                        quote_spanned! { f.span() => let #field_name = (*picolimbo_proto::UnprefixedByteArray::decode(read, ver)?.0).to_owned(); }
                    } else if attrs.iter().any(|attr| attr.meta.path().is_ident("prefixed")) {
                        let parsed = parse_pfx_type(attrs);
                        let generics = &f.ty;
                        quote_spanned! { f.span() => let #field_name: #generics = (*<#parsed>::decoding(read, ver)?.0).to_owned(); }
                    } else {
                        quote_spanned! { f.span() => let #field_name = <#ty>::decode(read, ver)?; }
                    }
                });
                let mut idx = 0;
                let self_builder = unnamed.unnamed.iter().map(|f| {
                    let field_name = Ident::new(&format!("field_{idx}"), f.span());
                    idx += 1;
                    quote_spanned! { f.span() => #field_name, }
                });
                Ok(quote! {
                    #[allow(unused_imports)]
                    use picolimbo_proto::ArrayPrefix;

                    #(#recurse)*;
                    Ok(Self(#(#self_builder)*))
                })
            }
            syn::Fields::Unit => Ok(quote!(Self)),
        },
        Data::Enum(dt) => Err(syn::Error::new_spanned(
            dt.enum_token,
            "Enums are not supported for Decodeable",
        )),
        Data::Union(dt) => Err(syn::Error::new_spanned(
            dt.union_token,
            "Unions are not supported for Decodeable",
        )),
    };
    qt
}
