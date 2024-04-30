use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, spanned::Spanned, Error, Item, ItemEnum, ItemStruct, Result};

pub fn expand_attribute_data(input: TokenStream) -> Result<TokenStream> {
    let item: Item = parse2(input.clone())?;
    match item {
        Item::Struct(item_struct) => expand_item_struct(input, item_struct),
        Item::Enum(item_enum) => expand_item_enum(input, item_enum),
        _others => Err(Error::new(input.span(), "Expected `struct` or `enum`")),
    }
}

pub fn expand_item_struct(input: TokenStream, item: ItemStruct) -> Result<TokenStream> {
    let ident = &item.ident;

    Ok(quote! {
        #input
    })
}

pub fn expand_item_enum(input: TokenStream, item: ItemEnum) -> Result<TokenStream> {
    Ok(quote! {
        #input
    })
}
