use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Data, DataEnum, DeriveInput, Error, Fields, Ident, Result, Variant};

pub fn expand_derive_event(input: &mut DeriveInput) -> Result<TokenStream> {
    let enum_ident = &input.ident;
    let data = &input.data;

    match data {
        Data::Enum(data_enum) => expand_enum(&enum_ident, data_enum),
        _others => Err(Error::new(
            input.span(),
            "Only `enum` type can derive `Event`",
        )),
    }
}

pub fn expand_enum(enum_ident: &Ident, data_enum: &DataEnum) -> Result<TokenStream> {
    let enum_variants = parse_data_enum(data_enum)?;

    Ok(quote! {
        impl_event(enum_ident, data_enum);
    })
}

pub fn parse_data_enum<'a>(data_enum: &'a DataEnum) -> Result<Vec<&'a EnumVariant>> {
    let enum_variants = data_enum
        .variants
        .iter()
        .map(|variant| EnumVariant::try_from(variant))
        .collect();
    Ok(vec![])
}

pub struct EnumVariant<'a> {
    name: &'a Ident,
    field: &'a Option<Fields>,
}

impl<'a> TryFrom<&'a Variant> for EnumVariant<'a> {
    type Error = Error;

    fn try_from(value: &'a Variant) -> std::prelude::v1::Result<Self, Self::Error> {
        if let (Fields::Named(named_field)) = value.fields {
            Err(Error(
                named_field.span(),
                "Enum variant must not be named struct",
            ))
        } else {
            Ok(Self {
                name: &value.ident,
                field: &value.fields,
            })
        }
    }
}

// pub fn check_enum_variant_fields(data_enum: &DataEnum) -> Result<()> {
//     let fields = data_enum.variants.iter().map(|variant| &variant.fields);
//     for field in fields {
//         if let Fields::Named(named_field) = field {
//             return Err(Error::new(
//                 named_field.span(),
//                 "Enum variant must not be named struct",
//             ));
//         }
//     }
//     Ok(())
// }

// pub struct EnumVariant {
//     name: Ident,
//     field: Option<Fields>,
// }

// pub fn parse_data_enum(data_enum: DataEnum) -> Result<Vec<EnumVariant>> {
//     let results = data_enum.variants.into_iter().map(|variant| EnumVariant::try_from()).collect();
// }

// pub fn impl_event(enum_ident: &Ident, data_enum: &DataEnum) -> TokenStream {
//     quote! {
//         impl sequencer_framework::Event for #enum_ident {
//             fn id(&self) -> &'static str {
//                 match self {
//                     #(Self::)
//                 }
//             }
//         }
//     }
// }
