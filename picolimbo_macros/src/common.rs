use syn::{GenericParam, Generics, TypeParamBound};

pub fn add_trait_bounds(mut generics: Generics, qt: TypeParamBound) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(qt.clone());
        }
    }
    generics
}
