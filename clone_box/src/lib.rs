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

use syn::punctuated::Punctuated;
use syn::token::{Colon, Where};
use syn::{TraitItem, ImplItem, PredicateType, TraitBound, WhereClause, WherePredicate, Type};

#[proc_macro_attribute]
pub fn clone_box(_attribute: TokenStream, input: TokenStream) -> TokenStream
{
    let item: syn::Item = syn::parse(input).expect("Unable to parse input tokenstream");

    match item
    {
        syn::Item::Trait(item_trait) => clone_box_trait(item_trait, false),
        syn::Item::Impl(item_impl) => clone_box_impl(item_impl, false),
        _ => panic!("Macro 'clone_box' can only be used on traits and trait-implementations")
    }
}

#[proc_macro_attribute]
pub fn clone_box_test(_attribute: TokenStream, input: TokenStream) -> TokenStream
{
    let item: syn::Item = syn::parse(input).expect("Unable to parse input tokenstream");

    match item
    {
        syn::Item::Trait(item_trait) => clone_box_trait(item_trait, true),
        syn::Item::Impl(item_impl) => clone_box_impl(item_impl, true),
        _ => panic!("Macro 'clone_box' can only be used on traits and trait-implementations")
    }
}

fn clone_box_trait(mut item: syn::ItemTrait, test: bool) -> TokenStream
{
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let found = item.items.iter().any(|item| {
        match item
        {
            TraitItem::Fn(method) => {
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
        item.items.push(TraitItem::Fn(syn::parse(quote!{
            fn clone_box(self: &Self) -> Box<dyn #ident #ty_generics>;
        }.into()).expect("Invalid method syntax")))
    }


    let output = quote! {
        #item

        impl #impl_generics Clone for Box<dyn #ident #ty_generics> #where_clause
        {
            fn clone(&self) -> Self
            {
                <dyn #ident #ty_generics>::clone_box(&**self)
            }
        }
    };

    if test
    {
        panic!("{}", output.to_string());
    }

    // Hand the output tokens back to the compiler.
    TokenStream::try_from(output).expect("Invalid 'Clone<Box<dyn Self>>' implementation")
}

fn clone_box_impl(mut item: syn::ItemImpl, test: bool) -> TokenStream
{
    let (bang, path, _) = match &item.trait_
    {
        Some(trait_) => trait_,
        None => panic!("Macro 'clone_box' can only be used on traits and trait-implementations")
    };
    {
        if bang.is_some()
        {
            panic!("Macro 'clone_box' can only be used on traits and trait-implementations");
        }
    
        let found = item.items.iter().any(|item| {
            match item
            {
                ImplItem::Fn(method) => {
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

    let (impl_generics, ty_generics, mut where_clause) = &item.generics.split_for_impl();
    let self_ty = &item.self_ty;
    let slf: Type = syn::parse(quote!{Self}.into()).unwrap();

    let mut where_clause = match where_clause
    {
        Some(where_clause) => WhereClause
        {
            where_token: where_clause.where_token,
            predicates: Punctuated::from_iter(where_clause.predicates.iter()
                .map(|predicate| match predicate
                {
                    WherePredicate::Type(predicate_type) => WherePredicate::Type(PredicateType
                        {
                            lifetimes: predicate_type.lifetimes.clone(),
                            bounded_ty: if predicate_type.bounded_ty.clone().into_token_stream().to_string() == slf.clone().into_token_stream().to_string()
                            {
                                *self_ty.clone()
                            }
                            else
                            {
                                predicate_type.bounded_ty.clone()
                            },
                            colon_token: predicate_type.colon_token,
                            bounds: predicate_type.bounds.clone()
                        }),
                    _ => predicate.clone()
                }))
        },
        None => WhereClause {
            where_token: Where::default(),
            predicates: Punctuated::new()
        }
    };
    {
        let mut bounds = Punctuated::new();
        bounds.push(syn::TypeParamBound::Trait(TraitBound {
            paren_token: None,
            modifier: syn::TraitBoundModifier::None,
            lifetimes: None,
            path: path.clone()
        }));
        where_clause.predicates.push(syn::WherePredicate::Type(PredicateType {
            lifetimes: None,
            bounded_ty: *self_ty.clone(),
            colon_token: Colon::default(),
            bounds
        }));
    }

    let output = quote! {
        #item
        
        impl #impl_generics From<#self_ty> for Box<dyn #path>
        #where_clause
        {
            fn from(from: #self_ty) -> Self
            {
                Box::new(from)
            }
        }
    };
    
    if test
    {
        panic!("{}", output.to_string());
    }

    TokenStream::try_from(output).expect("Invalid 'clone_box' implementation")
}