#![allow(warnings)]

mod bytereader;
mod server;
mod constants;

use crate::server::{Server, Response, ServerResponse};
use std::io;
use tui::Terminal;
use tui::backend::TermionBackend;
use termion::raw::IntoRawMode;
use tui::widgets::{Widget, Block, Borders};
use tui::layout::{Layout, Constraint, Direction};

fn main() -> Result<(), io::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default()
            .title("Block")
            .borders(Borders::ALL);
        f.render_widget(block, size);
    })

    // let mut server = Server::bind("78.70.133.111:27015");
    // match server
    //     .get_server_info() {
    //         ServerResponse::Failure(err) => panic!("{}", err),
    //         ServerResponse::SuccessResponse(server_info) => {
    //             println!("Server name: {}", server_info.name);
    //             println!("Players: {}/{}", server_info.players, server_info.max_players);
    //         }
    //     }
    // Ok(());
}
