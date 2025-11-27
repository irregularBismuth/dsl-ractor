mod ast;
mod expand;
mod kw;
mod parse;
mod validate;

use expand::expand;
use parse::actor_args::parse_actor_args;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{DeriveInput, parse_macro_input, spanned::Spanned};
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
///
/// This macro dramatically reduces boilerplate when implementing the `ractor::Actor` trait.
/// It automatically generates the trait implementation with proper type definitions and
/// method signatures, delegating to user-defined helper methods.
///
/// # Arguments
///
/// The macro accepts three named arguments:
/// - `msg`: The message type your actor will receive
/// - `state`: The state type your actor will maintain
/// - `args`: The arguments type used to initialize your actor (defaults to `()`)
///
/// # Example
///
/// ```rust,ignore
/// use dsl_ractor::actor;
/// use ractor::Actor;
///
/// #[derive(Debug, Clone)]
/// enum CounterMsg {
///     Increment,
///     Print,
/// }
///
/// #[actor(msg = CounterMsg, state = i32, args = i32)]
/// struct CounterActor;
/// ```
///
/// # What This Generates (and why helpers exist)
///
/// The macro expands into a complete `Actor` trait implementation:
/// - The trait methods delegate to helper methods on your type.
/// - You provide those helpers via the [`actor_pre_start!`] and [`actor_handle!`] macros.
///
/// The helpers are required because a proc macro attached to a struct cannot also inject
/// items into a separate `impl Actor for ...` block. Delegation keeps the API ergonomic
/// while satisfying Rust's macro hygiene rules.
///
/// ```rust,ignore
/// // Your original struct definition
/// struct CounterActor;
///
/// // Generated code (simplified):
/// #[async_trait]  // only with "async-trait" feature
/// impl Actor for CounterActor {
///     type Msg = CounterMsg;
///     type State = i32;
///     type Arguments = i32;
///
///     async fn pre_start(
///         &self,
///         myself: ActorRef<Self::Msg>,
///         args: Self::Arguments
///     ) -> Result<Self::State, ActorProcessingErr> {
///         self.on_start(myself, args).await
///     }
///
///     async fn handle(
///         &self,
///         myself: ActorRef<Self::Msg>,
///         msg: Self::Msg,
///         state: &mut Self::State
///     ) -> Result<(), ActorProcessingErr> {
///         self.handle_msg(myself, msg, state).await
///     }
/// }
/// ```
///
/// # Comparison: Before vs After
///
/// **Without this macro (raw Ractor):**
/// ```rust,ignore
/// use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait};
///
/// #[derive(Debug, Clone)]
/// enum CounterMsg {
///     Increment,
///     Print,
/// }
///
/// struct CounterActor;
///
/// #[async_trait]
/// impl Actor for CounterActor {
///     type Msg = CounterMsg;
///     type State = i32;
///     type Arguments = i32;
///
///     async fn pre_start(
///         &self,
///         _myself: ActorRef<Self::Msg>,
///         args: Self::Arguments,
///     ) -> Result<Self::State, ActorProcessingErr> {
///         Ok(args)  // 30+ lines of boilerplate just to get here!
///     }
///
///     async fn handle(
///         &self,
///         _myself: ActorRef<Self::Msg>,
///         message: Self::Msg,
///         state: &mut Self::State,
///     ) -> Result<(), ActorProcessingErr> {
///         match message {
///             CounterMsg::Increment => {
///                 *state += 1;
///                 Ok(())
///             }
///             CounterMsg::Print => {
///                 println!("Count: {}", state);
///                 Ok(())
///             }
///         }
///     }
/// }
/// ```
///
/// **With this macro:**
/// ```rust,ignore
/// use dsl_ractor::{actor, actor_pre_start, actor_handle};
///
/// #[derive(Debug, Clone)]
/// enum CounterMsg {
///     Increment,
///     Print,
/// }
///
/// #[actor(msg = CounterMsg, state = i32, args = i32)]
/// struct CounterActor;
///
/// impl CounterActor {
///     actor_pre_start!(Ok(args));
///
///     actor_handle!({
///         match msg {
///             CounterMsg::Increment => {
///                 *state += 1;
///                 Ok(())
///             }
///             CounterMsg::Print => {
///                 println!("Count: {}", state);
///                 Ok(())
///             }
///         }
///     });
/// }
/// ```
///
/// Notice how the macro:
/// - Eliminates the need to write `impl Actor for ...`
/// - No need to manually define associated types
/// - No need to write method signatures with complex return types
/// - Just focus on your actor's logic!
///
/// # Feature Flags
///
/// - `async-trait`: When enabled, uses the `async_trait` crate for trait methods.
///   When disabled, uses `impl Future` return type position impl trait (RPIT).
///
/// # See Also
///
/// - [`actor_pre_start!`] - Macro for defining the initialization logic
/// - [`actor_handle!`] - Macro for defining message handling logic
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

/// Procedural macro to define the actor's initialization logic.
///
/// This macro generates the `on_start()` helper method that is called by the
/// `Actor::pre_start()` trait method generated by [`#[actor]`](macro@actor).
///
/// # Purpose
///
/// The `pre_start` phase is where you initialize your actor's state from the
/// provided arguments. This happens once when the actor is spawned, before it
/// starts processing messages.
///
/// # Arguments
///
/// Accepts either a block or expression that evaluates to:
/// `Result<State, ActorProcessingErr>`
///
/// The following variables are available in scope:
/// - `myself`: `ActorRef<Self::Msg>` - Reference to this actor
/// - `args`: `Self::Arguments` - The arguments passed to `Actor::spawn()`
///
/// # Example
///
/// ```rust,ignore
/// use dsl_ractor::{actor, actor_pre_start, actor_handle};
///
/// #[actor(msg = String, state = Vec<String>, args = usize)]
/// struct LogActor;
///
/// impl LogActor {
///     // Simple expression: initialize empty vec with capacity
///     actor_pre_start!(Ok(Vec::with_capacity(args)));
///
///     // Or use a block for more complex initialization:
///     actor_pre_start!({
///         println!("Actor {:?} starting with capacity {}", myself, args);
///         let buffer = Vec::with_capacity(args);
///         Ok(buffer)
///     });
///
///     actor_handle!({
///         state.push(msg);
///         Ok(())
///     });
/// }
/// ```
///
/// # What This Generates
///
/// **Your code:**
/// ```rust,ignore
/// impl MyActor {
///     actor_pre_start!(Ok(args));
/// }
/// ```
///
/// **Expands to (with `async-trait` feature):**
/// ```rust,ignore
/// impl MyActor {
///     pub async fn on_start(
///         &self,
///         myself: ractor::ActorRef<<Self as ractor::Actor>::Msg>,
///         args: <Self as ractor::Actor>::Arguments,
///     ) -> Result<<Self as ractor::Actor>::State, ractor::ActorProcessingErr> {
///         Ok(args)
///     }
/// }
/// ```
///
/// **Expands to (without `async-trait` feature):**
/// ```rust,ignore
/// impl MyActor {
///     pub fn on_start(
///         &self,
///         myself: ractor::ActorRef<<Self as ractor::Actor>::Msg>,
///         args: <Self as ractor::Actor>::Arguments,
///     ) -> impl Future<Output = Result<<Self as ractor::Actor>::State, ractor::ActorProcessingErr>> + Send {
///         async move {
///             Ok(args)
///         }
///     }
/// }
/// ```
///
/// # Comparison: Before vs After
///
/// **Without this macro (raw Ractor):**
/// ```rust,ignore
/// #[async_trait]
/// impl Actor for MyActor {
///     type Msg = String;
///     type State = Vec<String>;
///     type Arguments = usize;
///
///     async fn pre_start(
///         &self,
///         myself: ActorRef<Self::Msg>,
///         args: Self::Arguments,
///     ) -> Result<Self::State, ActorProcessingErr> {
///         // Finally! Your actual logic:
///         Ok(Vec::with_capacity(args))
///     }
///     // ... handle method ...
/// }
/// ```
///
/// **With this macro:**
/// ```rust,ignore
/// #[actor(msg = String, state = Vec<String>, args = usize)]
/// struct MyActor;
///
/// impl MyActor {
///     actor_pre_start!(Ok(Vec::with_capacity(args)));
///     // ... actor_handle! ...
/// }
/// ```
///
/// Reduces **~15 lines** of boilerplate to **1 line** of actual logic!
///
/// # Architecture Note
///
/// This macro generates a helper method `on_start()` rather than the trait method
/// `pre_start()` directly because Rust proc macros running inside `impl MyActor`
/// cannot modify the separate `impl Actor for MyActor` block generated by `#[actor]`.
/// The trait method delegates to this helper method to bridge the gap.
///
/// # See Also
///
/// - [`#[actor]`](macro@actor) - Must be used first to set up the trait impl
/// - [`actor_handle!`] - For defining message handling logic
#[proc_macro]
pub fn actor_pre_start(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let body = parse_block_or_expr!(input);

    #[cfg(feature = "async-trait")]
    let tokens = quote::quote! {
        pub async fn on_start(
            &self,
            myself: ractor::ActorRef<<Self as ractor::Actor>::Msg>,
            args: <Self as ractor::Actor>::Arguments,
        ) -> ::core::result::Result<(<Self as ractor::Actor>::State), ractor::ActorProcessingErr> {
            #body
        }
    };

    #[cfg(not(feature = "async-trait"))]
    let tokens = quote::quote! {
        pub fn on_start(
            &self,
            myself: ractor::ActorRef<<Self as ractor::Actor>::Msg>,
            args: <Self as ractor::Actor>::Arguments,
        ) -> impl ::core::future::Future<
            Output=::core::result::Result<(<Self as ractor::Actor>::State), ractor::ActorProcessingErr>
        > + Send {
            async move {
                #body
            }
        }
    };

    tokens.into()
}

/// Procedural macro to define the actor's message handling logic.
///
/// This macro generates the `handle_msg()` helper method that is called by the
/// `Actor::handle()` trait method generated by [`#[actor]`](macro@actor).
///
/// # Purpose
///
/// This is the core of your actor - the logic that processes incoming messages
/// and updates state. This method is called repeatedly for each message the actor
/// receives during its lifetime.
///
/// # Arguments
///
/// Accepts either a block or expression that evaluates to:
/// `Result<(), ActorProcessingErr>`
///
/// The following variables are available in scope:
/// - `myself`: `ActorRef<Self::Msg>` - Reference to this actor (for sending messages to self)
/// - `msg`: `Self::Msg` - The incoming message to process
/// - `state`: `&mut Self::State` - Mutable reference to the actor's state
///
/// # Example
///
/// ```rust,ignore
/// use dsl_ractor::{actor, actor_pre_start, actor_handle};
/// use ractor::cast;
///
/// #[derive(Debug)]
/// enum CounterMsg {
///     Increment,
///     Decrement,
///     Reset,
///     Print,
/// }
///
/// #[actor(msg = CounterMsg, state = i32, args = i32)]
/// struct CounterActor;
///
/// impl CounterActor {
///     actor_pre_start!(Ok(args));
///
///     // Pattern match on messages and mutate state
///     actor_handle!({
///         match msg {
///             CounterMsg::Increment => {
///                 *state += 1;
///                 println!("Counter: {}", state);
///                 Ok(())
///             }
///             CounterMsg::Decrement => {
///                 *state -= 1;
///                 Ok(())
///             }
///             CounterMsg::Reset => {
///                 *state = 0;
///                 // Send messages to self
///                 cast!(myself, CounterMsg::Print)?;
///                 Ok(())
///             }
///             CounterMsg::Print => {
///                 println!("Current value: {}", state);
///                 Ok(())
///             }
///         }
///     });
/// }
/// ```
///
/// # What This Generates
///
/// **Your code:**
/// ```rust,ignore
/// impl MyActor {
///     actor_handle!({
///         match msg {
///             Msg::Ping => Ok(()),
///             Msg::Stop => myself.stop(None),
///         }
///     });
/// }
/// ```
///
/// **Expands to (with `async-trait` feature):**
/// ```rust,ignore
/// impl MyActor {
///     pub async fn handle_msg(
///         &self,
///         myself: ractor::ActorRef<<Self as ractor::Actor>::Msg>,
///         msg: <Self as ractor::Actor>::Msg,
///         state: &mut <Self as ractor::Actor>::State,
///     ) -> Result<(), ractor::ActorProcessingErr> {
///         match msg {
///             Msg::Ping => Ok(()),
///             Msg::Stop => myself.stop(None),
///         }
///     }
/// }
/// ```
///
/// **Expands to (without `async-trait` feature):**
/// ```rust,ignore
/// impl MyActor {
///     pub fn handle_msg(
///         &self,
///         myself: ractor::ActorRef<<Self as ractor::Actor>::Msg>,
///         msg: <Self as ractor::Actor>::Msg,
///         state: &mut <Self as ractor::Actor>::State,
///     ) -> impl Future<Output = Result<(), ractor::ActorProcessingErr>> + Send {
///         async move {
///             match msg {
///                 Msg::Ping => Ok(()),
///                 Msg::Stop => myself.stop(None),
///             }
///         }
///     }
/// }
/// ```
///
/// # Comparison: Before vs After
///
/// **Without this macro (raw Ractor):**
/// ```rust,ignore
/// #[async_trait]
/// impl Actor for PingPongActor {
///     type Msg = PingPongMsg;
///     type State = usize;
///     type Arguments = usize;
///
///     async fn pre_start(
///         &self,
///         _myself: ActorRef<Self::Msg>,
///         args: Self::Arguments,
///     ) -> Result<Self::State, ActorProcessingErr> {
///         Ok(args)
///     }
///
///     async fn handle(
///         &self,
///         myself: ActorRef<Self::Msg>,
///         message: Self::Msg,
///         state: &mut Self::State,
///     ) -> Result<(), ActorProcessingErr> {
///         //Your actual message handling logic:
///         match message {
///             PingPongMsg::Ping => {
///                 println!("Ping! Count: {}", state);
///                 *state -= 1;
///                 if *state > 0 {
///                     cast!(myself, PingPongMsg::Pong)?;
///                 }
///                 Ok(())
///             }
///             PingPongMsg::Pong => {
///                 println!("Pong! Count: {}", state);
///                 cast!(myself, PingPongMsg::Ping)?;
///                 Ok(())
///             }
///         }
///     }
/// }
/// ```
///
/// **With this macro:**
/// ```rust,ignore
/// #[actor(msg = PingPongMsg, state = usize, args = usize)]
/// struct PingPongActor;
///
/// impl PingPongActor {
///     actor_pre_start!(Ok(args));
///
///     actor_handle!({
///         match msg {
///             PingPongMsg::Ping => {
///                 println!("Ping! Count: {}", state);
///                 *state -= 1;
///                 if *state > 0 {
///                     cast!(myself, PingPongMsg::Pong)?;
///                 }
///                 Ok(())
///             }
///             PingPongMsg::Pong => {
///                 println!("Pong! Count: {}", state);
///                 cast!(myself, PingPongMsg::Ping)?;
///                 Ok(())
///             }
///         }
///     });
/// }
/// ```
///
/// Reduces **~25 lines** of repetitive trait boilerplate to just the essential logic!
///
/// # Architecture Note
///
/// This macro generates a helper method `handle_msg()` rather than the trait method
/// `handle()` directly because Rust proc macros running inside `impl MyActor` cannot
/// modify the separate `impl Actor for MyActor` block generated by `#[actor]`.
/// The trait method delegates to this helper method to bridge the gap.
///
/// # See Also
///
/// - [`#[actor]`](macro@actor) - Must be used first to set up the trait impl
/// - [`actor_pre_start!`] - For defining initialization logic
#[proc_macro]
pub fn actor_handle(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let body = parse_block_or_expr!(input);

    #[cfg(feature = "async-trait")]
    let tokens = quote::quote! {
        pub async fn handle_msg(
            &self,
            myself: ractor::ActorRef<<Self as ractor::Actor>::Msg>,
            msg: <Self as ractor::Actor>::Msg,
            state: &mut <Self as ractor::Actor>::State,
        ) -> ::core::result::Result<(), ractor::ActorProcessingErr> {
            #body
        }
    };

    #[cfg(not(feature = "async-trait"))]
    let tokens = quote::quote! {
        pub fn handle_msg(
            &self,
            myself: ractor::ActorRef<<Self as ractor::Actor>::Msg>,
            msg: <Self as ractor::Actor>::Msg,
            state: &mut <Self as ractor::Actor>::State,
        ) -> impl ::core::future::Future<
            Output=::core::result::Result<(), ractor::ActorProcessingErr>
        > + Send {
            async move {
                #body
            }
        }
    };

    tokens.into()
}
