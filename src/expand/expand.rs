use crate::ast::args::ValidatedActorArgs;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub(crate) fn expand(input: &DeriveInput, v: &ValidatedActorArgs) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let msg = &v.msg;
    let state = &v.state;
    let args_ty = &v.args;

    quote! {
        #input

        #[allow(unexpected_cfgs)]
        #[cfg(feature="async-trait")]
        #[::ractor::async_trait]
        impl #impl_generics ::ractor::Actor for #name #ty_generics #where_clause {
            type Msg = #msg;
            type State = #state;
            type Arguments = #args_ty;

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
        }

        #[allow(unexpected_cfgs)]
        #[cfg(not(feature = "async-trait"))]
        impl #impl_generics ::ractor::Actor for #name #ty_generics #where_clause {
            type Msg = #msg;
            type State = #state;
            type Arguments = #args_ty;

            fn pre_start(
                &self,
                myself: ::ractor::ActorRef<Self::Msg>,
                args: Self::Arguments
            ) -> impl ::core::future::Future<Output=::core::result::Result<Self::State, ::ractor::ActorProcessingErr>> + Send {
                self.on_start(myself, args)
            }

            fn handle(
                &self,
                myself: ::ractor::ActorRef<Self::Msg>,
                msg: Self::Msg,
                state: &mut Self::State
            ) -> impl ::core::future::Future<Output = ::core::result::Result<(), ::ractor::ActorProcessingErr>> + Send {
                self.handle_msg(myself, msg, state)
            }
        }
    }
}
