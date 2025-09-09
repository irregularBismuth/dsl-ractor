use crate::ast::{args::ActorArgsRaw, args::ValidatedActorArgs};
use syn::{parse_quote, Error, Result, Type};

fn require_field<T>(field: Option<T>, at: proc_macro2::Span, msg: &str) -> Result<T> {
    field.ok_or_else(|| Error::new(at, msg))
}

pub(crate) fn validate_actor_args(raw: ActorArgsRaw) -> Result<ValidatedActorArgs> {
    let msg: Type = require_field(raw.msg, raw.span, "missing `msg=...`")?;
    let state: Type = require_field(raw.state, raw.span, "missing `state=...`")?;
    let args: Type = raw.args.unwrap_or_else(|| parse_quote! { () });

    Ok(ValidatedActorArgs {
        msg,
        state,
        args,
        span: raw.span,
    })
}
