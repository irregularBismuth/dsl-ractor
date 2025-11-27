use anyhow::Result;
use dsl_ractor::actor;
use ractor::{Actor, cast};

#[derive(Debug, Clone)]
enum CounterMsg {
    Increment,
    Decrement,
    Print,
}

#[actor(msg = CounterMsg, state = i16, args = i16)]
struct CounterActor;

impl CounterActor {
    // Initialize state from args
    dsl_ractor::actor_pre_start!({
        Ok(args) // state = args (i16)
    });

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
