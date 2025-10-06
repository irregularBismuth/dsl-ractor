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

/// Parses input as either a `Block` or an `Expr`.
/// Falls back to wrapping the `Expr` in a block if no block is found.
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

/// Attribute macro to define an `Actor`.
/// Parses and validates arguments, then expands into the actor implementation.
/// Emits a compile error on invalid input.
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

/// Procedural macro to define the actor's `on_start` handler.
/// Expands the given block or expression into the async `on_start` method.
#[allow(unexpected_cfgs)]
#[proc_macro]
pub fn actor_pre_start(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let body = parse_block_or_expr!(input);
    quote::quote! {
        #[allow(unexpected_cfgs)]
        #[cfg(feature="async-trait")]
        pub async fn on_start(
            &self,
            myself: ractor::ActorRef<<Self as ractor::Actor>::Msg>,
            args: <Self as ractor::Actor>::Arguments,
        ) -> ::core::result::Result<(<Self as ractor::Actor>::State), ractor::ActorProcessingErr> {
            #body
        }

        #[allow(unexpected_cfgs)]
        #[cfg(not(feature="async-trait"))]
        pub fn on_start(
            &self,
            myself: ractor::ActorRef<<Self as ractor::Actor>::Msg>,
            args: <Self as ractor::Actor>::Arguments,
        ) -> impl ::core::future::Future<Output=::core::result::Result<(<Self as ractor::Actor>::State), ractor::ActorProcessingErr>> + Send {
            async move {
                #body
            }
        }
    }.into()
}

/// Procedural macro to define the actor's `handle_msg` handler.
/// Expands the given block or expression into the async `handle_msg` method.
#[allow(unexpected_cfgs)]
#[proc_macro]
pub fn actor_handle(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let body = parse_block_or_expr!(input);
    quote::quote! {
        #[allow(unexpected_cfgs)]
        #[cfg(feature="async-trait")]
        pub async fn handle_msg(
            &self,
            myself: ractor::ActorRef<<Self as ractor::Actor>::Msg>,
            msg: <Self as ractor::Actor>::Msg,
            state: &mut <Self as ractor::Actor>::State,
        ) -> ::core::result::Result<(), ractor::ActorProcessingErr> {
            #body
        }

        #[allow(unexpected_cfgs)]
        #[cfg(not(feature="async-trait"))]
        pub fn handle_msg(
            &self,
            myself: ractor::ActorRef<<Self as ractor::Actor>::Msg>,
            msg: <Self as ractor::Actor>::Msg,
            state: &mut <Self as ractor::Actor>::State,
        ) -> impl ::core::future::Future<Output=::core::result::Result<(), ractor::ActorProcessingErr>> + Send {
            async move {
                #body
            }
        }
    }.into()
}
