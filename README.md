# dsl-ractor

A simple procedural macro DSL for the [Ractor](https://crates.io/crates/ractor) actor framework.  
It reduces boilerplate so you can focus on your actor logic and ship faster.

Most Ractor actors repeat the same boilerplate: implementing `Actor`, defining `pre_start`,
and wiring `handle`. This crate generates it so you only write the logic that matters.

## Features
- `#[actor]` attribute to auto-generate `ractor::Actor` implementations
- `actor_pre_start!` for pre_start method
- `actor_handle!` for handle method

## Example
```rust
use anyhow::Result;
use dsl_ractor::actor;
use ractor::{cast, Actor, ActorRef};

#[derive(Debug, Clone)]
enum CounterMsg {
    Increment,
    Decrement,
    Print,
}

// Define the types `msg`, `state`, `args`
#[actor(msg = CounterMsg, state = i16, args = i16)]
pub struct CounterActor;

impl CounterActor {
    // Initialize state from args
    dsl_ractor::actor_pre_start!(
        Ok(args)
    );


    // Handle messages, mutate state
    dsl_ractor::actor_handle!({
        match msg {
            CounterMsg::Increment => {
                *state = state.saturating_add(1);
                Ok(())
            }
            CounterMsg::Decrement => {
                *state = state.saturating_sub(1);
                Ok(())
            }
            CounterMsg::Print => {
                println!("[Counter] value = {}", *state);
                Ok(())
            }
        }
    });
}

#[tokio::main]
async fn main() -> Result<()> {
    // Start counter at 5
    let (counter_ref, _handle) = Actor::spawn(None, CounterActor, 5_i16).await?;

    cast!(counter_ref, CounterMsg::Increment)?;
    cast!(counter_ref, CounterMsg::Increment)?;
    cast!(counter_ref, CounterMsg::Decrement)?;
    cast!(counter_ref, CounterMsg::Print)?;

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    Ok(())
}
```
