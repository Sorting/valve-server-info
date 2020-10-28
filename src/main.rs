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
    let mut server = Server::bind("178.236.67.44:27015");
    match server
        .get_server_info() {
            Response::Error(err) => panic!("{}", err),
            Response::Ok(server_info) => {
                println!("Server name: {}", server_info.name);
                println!("Players: {}/{}", server_info.players, server_info.max_players);
                println!("Map: {}", server_info.map);
                println!("Server Type: {:?}", server_info.server_type);
                println!("Environment: {:?}", server_info.environment);
                println!("Visibility: {:?}", server_info.server_visibility);
            }
        }    
}
