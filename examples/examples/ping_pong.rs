use dsl_ractor::{actor, actor_handle, actor_pre_start};
enum Msg {
    Ping,
    Pong,
    Exit,
}

#[actor(msg=Msg, state=usize, args=usize)]
struct Pinger;

impl Pinger {
    //Provide init value for the actor
    actor_pre_start!(Ok(args));

    actor_handle!({
        match msg {
            Msg::Ping => {
                println!("[Ping] {}", state);
                *state -= 1;
                ractor::cast!(myself, Msg::Pong).unwrap();
            }
            Msg::Pong => {
                println!("Sending [Pong] {}", state);
                if *state == 0 {
                    ractor::cast!(myself, Msg::Exit).unwrap();
                }

                ractor::cast!(myself, Msg::Ping).unwrap();
            }
            _ => {
                myself.stop(None);
            }
        }
        Ok(())
    });
}
#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let (tx, _) = ractor::Actor::spawn(None, Pinger, 10usize)
        .await
        .expect("Failed to start actor");

    ractor::cast!(tx, Msg::Ping).unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
}
