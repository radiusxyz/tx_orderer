mod sequencer_data;

use proc_macro::TokenStream;
use syn::Error;

#[proc_macro_attribute]
pub fn sequencer_data(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    sequencer_data::expand_sequencer_data(input.into())
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
