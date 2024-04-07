use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, spanned::Spanned, Error, Ident, Item, ItemEnum, ItemStruct, Result, Type};

pub fn expand_sequencer_data(input: TokenStream) -> Result<TokenStream> {
    let item: Item = parse2(input.clone())?;

    match item {
        Item::Struct(item_struct) => expand_item_struct(input, item_struct),
        Item::Enum(item_enum) => expand_item_enum(input, item_enum),
        _others => Err(Error::new(input.span(), "Expected `struct` or `enum`")),
    }
}

pub fn expand_item_struct(input: TokenStream, item_struct: ItemStruct) -> Result<TokenStream> {
    // eprintln!("{:#?}", item_struct);
    check_fields_trait_bound(item_struct);

    Ok(quote! {
        #[derive(Debug, sequencer_framework::serde::Deserialize, sequencer_framework::serde::Serialize)]
        #[serde(crate = "sequencer_framework::serde")]
        #input
    })
}

pub fn expand_item_enum(input: TokenStream, item_enum: ItemEnum) -> Result<TokenStream> {
    Ok(quote! {
        #[derive(Debug, sequencer_framework::serde::Deserialize, sequencer_framework::serde::Serialize)]
        #[serde(crate = "sequencer_framework::serde")]
        #input
    })
}

pub fn check_fields_trait_bound(item_struct: ItemStruct) {
    for field in item_struct.fields.iter() {
        if let Type::Path(ref type_path) = field.ty {
            eprintln!("{:#?}", type_path);
            // let ident = type_path.path.get_ident();
            // eprintln!("{:#?}", ident);
        }
    }

    // let field_types: Vec<&Ident> = item_struct
    //     .fields
    //     .iter()
    //     .filter_map(|field| {
    //         if let Type::Path(ref type_path) = field.ty {
    //             type_path.path.get_ident()
    //         } else {
    //             None
    //         }
    //     })
    //     .filter_map(|ident| )
    //     .collect();
}
