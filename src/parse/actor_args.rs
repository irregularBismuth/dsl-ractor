use crate::ast::args::ActorArgsRaw;
use crate::kw;
use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    Result, Token, Type,
};

enum ItemKey {
    Msg,
    State,
    Args,
}

macro_rules! define_parser {
      ($($kw:ident => $variant:ident),* $(,)?) => {
          impl Parse for Item {
              fn parse(input: ParseStream) -> Result<Self> {
                  let lookahead = input.lookahead1();
                  let key = $(
                      if lookahead.peek(kw::$kw) {
                          input.parse::<kw::$kw>()?;
                          ItemKey::$variant
                      } else
                  )* {
                      return Err(lookahead.error());
                  };

                  input.parse::<Token![=]>()?;
                  let val = input.parse()?;
                  Ok(Item { key, val })
              }
          }
      };
  }

struct Item {
    key: ItemKey,
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

    Ok(items.into_iter().fold(
        ActorArgsRaw::new(attr_span),
        |mut acc, Item { key, val }| {
            match key {
                ItemKey::Msg => acc.msg = Some(val),
                ItemKey::State => acc.state = Some(val),
                ItemKey::Args => acc.args = Some(val),
            }
            acc
        },
    ))
}
