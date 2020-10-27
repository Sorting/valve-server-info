mod bytereader;
mod server;
mod constants;

use crate::server::{Server, Response, ServerResponse};

fn main() {
    let mut server = Server::bind("78.70.133.111:27015");
    match server
        .get_server_info() {
            ServerResponse::Failure(err) => panic!("{}", err),
            ServerResponse::SuccessResponse(server_info) => {
                println!("Server name: {}", server_info.name);
                println!("Players: {}/{}", server_info.players, server_info.max_players);
            }
        }
}
