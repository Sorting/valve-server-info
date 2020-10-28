mod bytereader;
mod server;
mod constants;

use crate::server::{Server, Response};
use std::io;
use tui::Terminal;
use tui::backend::TermionBackend;
use termion::raw::IntoRawMode;
use tui::widgets::{Widget, Block, Borders};
use tui::layout::{Layout, Constraint, Direction};

fn main() {
    let mut server = Server::connect("178.236.67.44:27015");
    match server.get_server_info() {
        Response::Error(err) => panic!("Failed to fetch server info. {}", err),
        Response::Ok(server_info) => {
            println!("{:#?}", server_info);

            match server.get_players() {
                Response::Error(err) => panic!("Failed to fetch players. {}", err),
                Response::Ok(player_response) => {
                    println!("{:#?}", player_response);
                }
            }
        }
    }
}
