pub mod container;
pub mod metadata;

use container::ContainerType;
use metadata::Metadata;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Item, Result};

pub fn expand_attribute_data(metadata: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let metadata: Option<Metadata> = match metadata.is_empty() {
        true => None,
        false => Some(syn::parse2(metadata.clone())?),
    };
    let item: Item = syn::parse2(input.clone())?;
    let container = ContainerType::new(metadata, &item)?;
    let impl_blocks = container.impl_blocks();

    Ok(quote! {
        #[derive(Clone, Debug, tpm::serde::Deserialize, tpm::serde::Serialize)]
        #[serde(crate = "tpm::serde")]
        #input
        #impl_blocks
    })
}

pub trait AttributeData {
    fn fn_new(&self) -> TokenStream;

    fn fn_load(&self) -> Option<TokenStream>;

    fn fn_save(&self) -> Option<TokenStream>;

    fn fn_emit(&self) -> Option<TokenStream>;
}
