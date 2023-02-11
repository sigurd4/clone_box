#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

extern crate proc_macro;

use crate::proc_macro::TokenStream;

use quote::{quote, ToTokens};

use syn::token::Fn;
use syn::{DeriveInput, parse_macro_input, TraitItem, TraitItemMethod, Signature, Ident, ImplItem};

#[proc_macro_attribute]
pub fn clone_box(_attribute: TokenStream, input: TokenStream) -> TokenStream
{
    let item: syn::Item = syn::parse(input).expect("Unable to parse input tokenstream");

    match item
    {
        syn::Item::Trait(item_trait) => clone_box_trait(item_trait),
        syn::Item::Impl(item_impl) => clone_box_impl(item_impl),
        _ => panic!("Macro 'clone_box' can only be used on traits and trait-implementations")
    }
}

fn clone_box_trait(mut item: syn::ItemTrait) -> TokenStream
{
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let found = item.items.iter().any(|item| {
        match item
        {
            TraitItem::Method(method) => {
                method.sig.ident.to_string() == "clone_box"
            },
            _ => false
        }
    });
    
    let ident = &item.ident;
    
    if found
    {
        panic!("Method named 'clone_box' already declared in trait");
    }

    if !found
    {
        item.items.push(TraitItem::Method(syn::parse(quote!{
            fn clone_box(self: &Self) -> Box<dyn #ident #ty_generics>;
        }.into()).expect("Invalid method syntax")))
    }


    let output = quote! {
        #item

        impl #impl_generics Clone for Box<dyn #ident #ty_generics> #where_clause
        {
            fn clone(&self) -> Self
            {
                <#ident>::clone_box(self)
            }
        }
    };

    //panic!("{}", output.to_string());

    // Hand the output tokens back to the compiler.
    TokenStream::try_from(output).expect("Invalid 'Clone<Box<dyn Self>>' implementation")
}

fn clone_box_impl(mut item: syn::ItemImpl) -> TokenStream
{
    {
        let (bang, path, _) = match &item.trait_
        {
            Some(trait_) => trait_,
            None => panic!("Macro 'clone_box' can only be used on traits and trait-implementations")
        };
    
        if bang.is_some()
        {
            panic!("Macro 'clone_box' can only be used on traits and trait-implementations");
        }
    
        let found = item.items.iter().any(|item| {
            match item
            {
                ImplItem::Method(method) => {
                    method.sig.ident.to_string() == "clone_box"
                }
                _ => false
            }
        });
    
        if found
        {
            panic!("Method named 'clone_box' already in implementation");
        }
    
        item.items.push(syn::parse(quote!{
            fn clone_box(self: &Self) -> Box<dyn #path>
            {
                Box::new(self.clone())
            }
        }.into()).expect("Invalid clone_box implementation"));
    }
    let item = &item;
    
    let output = quote! {
        #item
    };

    TokenStream::try_from(output).expect("Invalid 'clone_box' implementation")
}