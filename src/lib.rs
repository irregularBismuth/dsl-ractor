mod ast;
mod expand;
mod kw;
mod parse;
mod validate;

use expand::expand::expand;
use parse::actor_args::parse_actor_args;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};
use validate::args::validate_actor_args;

macro_rules! parse_block_or_expr {
    ($input:expr) => {
        if let Ok(block) = syn::parse::<syn::Block>($input.clone()) {
            block
        } else {
            let expr: syn::Expr = syn::parse($input).expect("expected block or expression");
            syn::parse_quote!({ #expr })
        }
    };
}

#[proc_macro_attribute]
pub fn actor(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let attr_ts: TokenStream2 = attr.into();
    let out = || -> syn::Result<_> {
        let raw = parse_actor_args(input.span(), attr_ts)?;
        let val = validate_actor_args(raw)?;
        Ok(expand(&input, &val))
    }();

    match out {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn actor_pre_start(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let body = parse_block_or_expr!(input);
    quote::quote! {
        pub async fn on_start(
            &self,
            myself: ractor::ActorRef<<Self as ractor::Actor>::Msg>,
            args: <Self as ractor::Actor>::Arguments,
        ) -> ::core::result::Result<(<Self as ractor::Actor>::State), ractor::ActorProcessingErr> {
            let this = self;
            #body
        }
    }
    .into()
}

#[proc_macro]
pub fn actor_handle(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let body = parse_block_or_expr!(input);
    quote::quote! {
        pub async fn handle_msg(
            &self,
            myself: ractor::ActorRef<<Self as ractor::Actor>::Msg>,
            msg: <Self as ractor::Actor>::Msg,
            state: &mut <Self as ractor::Actor>::State,
        ) -> ::core::result::Result<(), ractor::ActorProcessingErr> {
            let this = self;
            #body
        }
    }
    .into()
}
