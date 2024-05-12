pub mod container_enum;
pub mod container_struct;

use container_enum::ContainerEnum;
use container_struct::ContainerStruct;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Error, Generics, Ident, Item, Result};

use crate::attribute_data::{metadata::Metadata, AttributeData};

pub trait Container {
    fn generics(&self) -> &Generics;

    fn ident(&self) -> &Ident;
}

pub enum ContainerType<'ast> {
    Enum(ContainerEnum<'ast>),
    Struct(ContainerStruct<'ast>),
}

impl<'ast> ContainerType<'ast> {
    pub fn new(metadata: Option<Metadata>, item: &'ast Item) -> Result<Self> {
        match item {
            Item::Enum(item_enum) => Ok(Self::Enum(ContainerEnum::new(metadata, item_enum))),
            Item::Struct(item_struct) => {
                Ok(Self::Struct(ContainerStruct::new(metadata, item_struct)))
            }
            _others => Err(Error::new(
                item.span(),
                "`data` attribute must be used on either `enum` or `struct` ",
            )),
        }
    }

    pub fn impl_blocks(&self) -> TokenStream {
        let generics = self.generics();
        let ident = self.ident();
        let fn_new = self.fn_new();
        let fn_load = self.fn_load();
        let fn_save = self.fn_save();
        let fn_emit = self.fn_emit();

        quote! {
            impl #generics #ident #generics {
                #fn_new
                #fn_load
                #fn_save
                #fn_emit
            }
        }
    }
}

impl<'ast> Container for ContainerType<'ast> {
    fn generics(&self) -> &Generics {
        match self {
            Self::Enum(container) => container.generics(),
            Self::Struct(container) => container.generics(),
        }
    }

    fn ident(&self) -> &Ident {
        match self {
            Self::Enum(container) => container.ident(),
            Self::Struct(container) => container.ident(),
        }
    }
}

impl<'ast> AttributeData for ContainerType<'ast> {
    fn fn_new(&self) -> TokenStream {
        match self {
            Self::Enum(container) => container.fn_new(),
            Self::Struct(container) => container.fn_new(),
        }
    }

    fn fn_load(&self) -> Option<TokenStream> {
        match self {
            Self::Enum(container) => container.fn_load(),
            Self::Struct(container) => container.fn_load(),
        }
    }

    fn fn_save(&self) -> Option<TokenStream> {
        match self {
            Self::Enum(container) => container.fn_save(),
            Self::Struct(container) => container.fn_save(),
        }
    }

    fn fn_emit(&self) -> Option<TokenStream> {
        match self {
            Self::Enum(container) => container.fn_emit(),
            Self::Struct(container) => container.fn_emit(),
        }
    }
}
