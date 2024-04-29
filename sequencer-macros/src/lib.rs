mod data;

use proc_macro::TokenStream;
use syn::Error;

#[proc_macro_attribute]
pub fn data(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    data::expand_attribute_data(input.into())
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

// #[proc_macro_derive(Data)]
// pub fn derive_data(input: TokenStream) -> TokenStream {
//     let mut input = parse_macro_input!(input as DeriveInput);
//     data::expand_derive_data(&mut input)
//         .unwrap_or_else(Error::into_compile_error)
//         .into()
// }
