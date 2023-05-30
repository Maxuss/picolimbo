use syn::{Attribute, GenericParam, Generics, TypeParamBound};

pub fn add_trait_bounds(mut generics: Generics, qt: TypeParamBound) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(qt.clone());
        }
    }
    generics
}

pub fn parse_pfx_type(attrs: &[Attribute]) -> syn::Type {
    let attr = attrs
        .iter()
        .filter(|attr| attr.meta.path().is_ident("prefixed"))
        .last()
        .unwrap();
    let list = attr.meta.require_list().unwrap();
    let tokens: proc_macro::TokenStream = list.tokens.clone().into();
    syn::parse::<syn::Type>(tokens).unwrap()
}
