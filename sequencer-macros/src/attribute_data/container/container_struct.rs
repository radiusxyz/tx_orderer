use core::iter::Map;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Iter, Field, Generics, Ident, ItemStruct};

use crate::attribute_data::{container::Container, metadata::Metadata, AttributeData};

pub enum ContainerStruct<'ast> {
    Named(NamedStruct<'ast>),
    Tuple(TupleStruct<'ast>),
}

impl<'ast> ContainerStruct<'ast> {
    pub fn new(metadata: Option<Metadata>, item_struct: &'ast ItemStruct) -> Self {
        match item_struct.semi_token {
            Some(_) => Self::Tuple(TupleStruct::new(metadata, item_struct)),
            None => Self::Named(NamedStruct::new(metadata, item_struct)),
        }
    }
}

impl<'ast> Container for ContainerStruct<'ast> {
    fn generics(&self) -> &Generics {
        match self {
            Self::Named(container) => container.generics(),
            Self::Tuple(container) => container.generics(),
        }
    }

    fn ident(&self) -> &Ident {
        match self {
            Self::Named(container) => container.ident(),
            Self::Tuple(container) => container.ident(),
        }
    }
}

impl<'ast> AttributeData for ContainerStruct<'ast> {
    fn fn_new(&self) -> TokenStream {
        match self {
            Self::Named(container) => container.fn_new(),
            Self::Tuple(container) => container.fn_new(),
        }
    }

    fn fn_load(&self) -> Option<TokenStream> {
        match self {
            Self::Named(container) => container.fn_load(),
            Self::Tuple(container) => container.fn_load(),
        }
    }

    fn fn_save(&self) -> Option<TokenStream> {
        match self {
            Self::Named(container) => container.fn_save(),
            Self::Tuple(container) => container.fn_save(),
        }
    }

    fn fn_emit(&self) -> Option<TokenStream> {
        match self {
            Self::Named(container) => container.fn_emit(),
            Self::Tuple(container) => container.fn_emit(),
        }
    }
}

pub struct NamedStruct<'ast> {
    metadata: Option<Metadata>,
    item_struct: &'ast ItemStruct,
}

impl<'ast> NamedStruct<'ast> {
    pub fn new(metadata: Option<Metadata>, item_struct: &'ast ItemStruct) -> Self {
        Self {
            metadata,
            item_struct,
        }
    }
}

impl<'ast> Container for NamedStruct<'ast> {
    fn generics(&self) -> &Generics {
        &self.item_struct.generics
    }

    fn ident(&self) -> &Ident {
        &self.item_struct.ident
    }
}

impl<'ast> AttributeData for NamedStruct<'ast> {
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

pub struct TupleStruct<'ast> {
    metadata: Option<Metadata>,
    item_struct: &'ast ItemStruct,
}

impl<'ast> TupleStruct<'ast> {
    pub fn new(metadata: Option<Metadata>, item_struct: &'ast ItemStruct) -> Self {
        Self {
            metadata,
            item_struct,
        }
    }
}

impl<'ast> Container for TupleStruct<'ast> {
    fn generics(&self) -> &Generics {
        &self.item_struct.generics
    }

    fn ident(&self) -> &Ident {
        &self.item_struct.ident
    }
}

impl<'ast> AttributeData for TupleStruct<'ast> {
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
