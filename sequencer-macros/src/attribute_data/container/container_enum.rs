use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemEnum;

use crate::attribute_data::{container::Container, metadata::Metadata, AttributeData};

pub struct ContainerEnum<'ast> {
    metadata: Option<Metadata>,
    item_enum: &'ast ItemEnum,
}

impl<'ast> ContainerEnum<'ast> {
    pub fn new(metadata: Option<Metadata>, item_enum: &'ast ItemEnum) -> Self {
        Self {
            metadata,
            item_enum,
        }
    }
}

impl<'ast> Container for ContainerEnum<'ast> {
    fn generics(&self) -> &syn::Generics {
        &self.item_enum.generics
    }

    fn ident(&self) -> &syn::Ident {
        &self.item_enum.ident
    }
}

impl<'ast> AttributeData for ContainerEnum<'ast> {
    fn fn_new(&self) -> TokenStream {
        quote! {}
    }

    fn fn_load(&self) -> Option<TokenStream> {
        if let Some(metadata) = &self.metadata {
            let key_ident = metadata.key_ident();
            let key_type = metadata.key_type();
            Some(quote! {
                pub fn load(#(#key_ident: &#key_type,)*) {}
            })
        } else {
            None
        }
    }

    fn fn_save(&self) -> Option<TokenStream> {
        if let Some(metadata) = &self.metadata {
            let key_ident = metadata.key_ident();
            let key_type = metadata.key_type();
            Some(quote! {
                pub fn save(&self, #(#key_ident: &#key_type,)*) {}
            })
        } else {
            None
        }
    }

    fn fn_emit(&self) -> Option<TokenStream> {
        if let Some(metadata) = &self.metadata {
            let key_ident = metadata.key_ident();
            let key_type = metadata.key_type();
            Some(quote! {
                pub fn emit(&self, #(#key_ident: &#key_type,)*) {}
            })
        } else {
            None
        }
    }
}
