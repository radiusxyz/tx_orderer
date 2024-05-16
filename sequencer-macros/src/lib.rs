pub(crate) mod attribute_data;
pub(crate) mod util;
// pub(crate) mod event;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Error};

#[proc_macro_attribute]
pub fn data(metadata: TokenStream, input: TokenStream) -> TokenStream {
    attribute_data::expand_attribute_data(metadata.into(), input.into())
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

// #[proc_macro_derive(Event)]
// pub fn derive_event(input: TokenStream) -> TokenStream {
//     let mut input = parse_macro_input!(input as DeriveInput);
//     event::expand_derive_event(&mut input)
//         .unwrap_or_else(Error::into_compile_error)
//         .into()
// }
