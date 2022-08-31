use std::env;
use shikataganai_server::ecs::plugins::server::ShikataganaiServerAddress;
use shikataganai_server::spawn_server;

fn main() {
  let address: Option<String> = env::args().into_iter().nth(1);

  let address = match address {
    None => {
      ShikataganaiServerAddress { address: "127.0.0.1:8181".to_string() }
    }
    Some(address) => {
      ShikataganaiServerAddress { address }
    }
  };

  spawn_server(address);
}
