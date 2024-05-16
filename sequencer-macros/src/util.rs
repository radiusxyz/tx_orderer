use proc_macro2::TokenStream;
use quote::quote;

pub fn serde_path() -> TokenStream {
    quote! {
        #[serde(crate = "sequencer_framework::serde")]
    }
}

pub fn deserialize() -> TokenStream {
    quote! {
        sequencer_framework::serde::Deserialize
    }
}

pub fn serialize() -> TokenStream {
    quote! {
        sequencer_framework::serde::Serialize
    }
}
