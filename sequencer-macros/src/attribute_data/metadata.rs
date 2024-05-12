use core::iter::{Enumerate, Map};

use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::{Iter, Punctuated},
    spanned::Spanned,
    token::Paren,
    Ident, Result, Token, Type,
};

#[derive(Debug)]
#[allow(unused)]
pub struct Metadata {
    pub key: Ident,
    pub paren_token: Paren,
    pub values: Punctuated<Type, Token![,]>,
}

impl Parse for Metadata {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            key: input.parse()?,
            paren_token: parenthesized!(content in input),
            values: content.parse_terminated(Type::parse, Token![,])?,
        })
    }
}

impl Metadata {
    pub fn key_ident<'meta>(
        &'meta self,
    ) -> Map<Enumerate<Iter<Type>>, impl FnMut((usize, &'meta Type)) -> Ident> {
        self.values.iter().enumerate().map(|(key_index, key_type)| {
            let key_ident = format!("key_{}", key_index);
            Ident::new(&key_ident, key_type.span())
        })
    }

    pub fn key_type<'meta>(&'meta self) -> Map<Iter<Type>, impl FnMut(&'meta Type) -> &Type> {
        self.values.iter().map(|key_type| key_type)
    }
}
