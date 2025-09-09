use proc_macro2::Span;
use syn::Type;

#[derive(Clone)]
pub(crate) struct ActorArgsRaw {
    pub msg: Option<Type>,
    pub state: Option<Type>,
    pub args: Option<Type>,
    pub span: Span,
}

impl ActorArgsRaw {
    pub fn new(span: Span) -> Self {
        Self {
            msg: None,
            state: None,
            args: None,
            span,
        }
    }
}

#[non_exhaustive]
#[derive(Clone)]
pub(crate) struct ValidatedActorArgs {
    pub msg: Type,
    pub state: Type,
    pub args: Type,
    #[allow(dead_code)]
    pub span: Span,
}
