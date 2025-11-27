use crate::ast::args::ValidatedActorArgs;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, ImplGenerics, Type, TypeGenerics, WhereClause};

/// Public entry point used by the `#[actor]` attribute.
pub(crate) fn expand(input: &DeriveInput, v: &ValidatedActorArgs) -> TokenStream {
    let ctx = ExpansionContext::new(input, v);

    #[cfg(feature = "async-trait")]
    {
        async_trait::expand(&ctx)
    }

    #[cfg(not(feature = "async-trait"))]
    {
        rpit::expand(&ctx)
    }
}

/// Shared context that feeds both feature branches.
struct ExpansionContext<'a> {
    input: &'a DeriveInput,
    name: &'a Ident,
    impl_generics: ImplGenerics<'a>,
    ty_generics: TypeGenerics<'a>,
    where_clause: Option<&'a WhereClause>,
    msg: &'a Type,
    state: &'a Type,
    args_ty: &'a Type,
}

impl<'a> ExpansionContext<'a> {
    fn new(input: &'a DeriveInput, v: &'a ValidatedActorArgs) -> Self {
        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

        Self {
            input,
            name: &input.ident,
            impl_generics,
            ty_generics,
            where_clause,
            msg: &v.msg,
            state: &v.state,
            args_ty: &v.args,
        }
    }

    fn render(&self, impl_attrs: TokenStream, methods: TokenStream) -> TokenStream {
        let Self {
            input,
            name,
            impl_generics,
            ty_generics,
            where_clause,
            msg,
            state,
            args_ty,
        } = self;

        quote! {
            #input

            #impl_attrs
            impl #impl_generics ::ractor::Actor for #name #ty_generics #where_clause {
                type Msg = #msg;
                type State = #state;
                type Arguments = #args_ty;

                #methods
            }
        }
    }
}

/// `async-trait` feature branch.
#[cfg(feature = "async-trait")]
mod async_trait {
    use super::*;

    pub(super) fn expand(ctx: &ExpansionContext<'_>) -> TokenStream {
        let methods = quote! {
            async fn pre_start(
                &self,
                myself: ::ractor::ActorRef<Self::Msg>,
                args: Self::Arguments
            ) -> ::core::result::Result<Self::State, ::ractor::ActorProcessingErr> {
                self.on_start(myself, args).await
            }

            async fn handle(
                &self,
                myself: ::ractor::ActorRef<Self::Msg>,
                msg: Self::Msg,
                state: &mut Self::State
            ) -> ::core::result::Result<(), ::ractor::ActorProcessingErr> {
                self.handle_msg(myself, msg, state).await
            }
        };

        ctx.render(quote!(#[::ractor::async_trait]), methods)
    }
}

/// RPIT (impl Future) feature branch.
#[cfg(not(feature = "async-trait"))]
mod rpit {
    use super::*;

    pub(super) fn expand(ctx: &ExpansionContext<'_>) -> TokenStream {
        let methods = quote! {
            fn pre_start(
                &self,
                myself: ::ractor::ActorRef<Self::Msg>,
                args: Self::Arguments
            ) -> impl ::core::future::Future<
                Output = ::core::result::Result<Self::State, ::ractor::ActorProcessingErr>
            > + Send {
                self.on_start(myself, args)
            }

            fn handle(
                &self,
                myself: ::ractor::ActorRef<Self::Msg>,
                msg: Self::Msg,
                state: &mut Self::State
            ) -> impl ::core::future::Future<
                Output = ::core::result::Result<(), ::ractor::ActorProcessingErr>
            > + Send {
                self.handle_msg(myself, msg, state)
            }
        };

        ctx.render(TokenStream::new(), methods)
    }
}
