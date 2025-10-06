use crate::ast::args::ActorArgsRaw;
use crate::kw;
use proc_macro2::Span;
use syn::{
    Error, Result, Token, Type,
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    spanned::Spanned,
};

/// Keys supported in the parser (`msg`, `state`, `args`).
enum ItemKey {
    Msg,
    State,
    Args,
}

///Macro to generate a `Parse` impl for `Item`.
///Maps keywords to `ItemKey` variants and parses `key = value` pairs.
macro_rules! define_parser {
      ($($kw:ident => $variant:ident),* $(,)?) => {
          impl Parse for Item {
              fn parse(input: ParseStream) -> Result<Self> {
                  let lookahead = input.lookahead1();
                  let (key, key_span) = $(
                      if lookahead.peek(kw::$kw) {
                          let token: kw::$kw = input.parse()?;
                          (ItemKey::$variant, token.span())
                      } else
                  )* {
                      return Err(lookahead.error());
                  };

                  input.parse::<Token![=]>()?;
                  let val = input.parse()?;
                  Ok(Item { key, key_span, val })
              }
          }
      };
}

///Parsed `key = value` pair, where the key maps to an `ItemKey`
struct Item {
    key: ItemKey,
    key_span: Span,
    val: Type,
}

define_parser!(
    msg => Msg,
    state=> State,
    args=> Args
);

pub(crate) fn parse_actor_args(
    attr_span: Span,
    ts: proc_macro2::TokenStream,
) -> Result<ActorArgsRaw> {
    let parser = Punctuated::<Item, Token![,]>::parse_terminated;
    let items: Punctuated<Item, Token![,]> = parser.parse2(ts)?;

    items.into_iter().try_fold(
        ActorArgsRaw::new(attr_span),
        |mut acc, Item { key, key_span, val }| {
            let (slot, err_msg) = match key {
                ItemKey::Msg => (&mut acc.msg, "duplicate `msg` argument"),
                ItemKey::State => (&mut acc.state, "duplicate `state` argument"),
                ItemKey::Args => (&mut acc.args, "duplicate `args` argument"),
            };

            slot.replace(val)
                .map_or(Ok(acc), |_| Err(Error::new(key_span, err_msg)))
        },
    )
}
