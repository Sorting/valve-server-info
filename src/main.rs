mod bytereader;
mod server;
mod ui;
mod constants;
mod util;

use crate::util::{
    event::{Event, Events},
};

use crate::server::{Server, Response, ServerInfo, PlayersResponse };
use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, TableState},
    Terminal,
};

pub struct StatefulTable{
    server: Server,
    server_info: Option<ServerInfo>,
    players_info: Option<PlayersResponse>,
    state: TableState,
    items: Vec<Vec<String>>,
}

impl StatefulTable {    
    fn new() -> StatefulTable {      

        StatefulTable {
            players_info: None,
            server_info: None,
            server: Server::connect("178.236.67.8:27015"),
            state: TableState::default(),
            items: vec![],            
        }
    }

    pub fn get_stats(&mut self) -> Result<(ServerInfo, PlayersResponse), String> {        
        match self.server.get_server_info() {
            Response::Error(err) => {                
                 Err(err)
            },
            Response::Ok(server_info) => {
                match self.server.get_players() {
                    Response::Error(err) => Err(err),
                    Response::Ok(player_response) => {
                        Ok((server_info, player_response))
                    }
                }
            }
        }
    }

    pub fn update(&mut self) {
        let (server_info, players_info) = (self.get_stats()).unwrap();
        
        let mut players_rows = vec![];

        for player in players_info.players.iter() {
            players_rows.push(
                vec![
                    player.name.clone(),
                    player.score.to_string(), 
                    format!("{}h, {:02}m, {:02}s", 
                        player.duration.num_hours(), 
                        player.duration.num_minutes()-(player.duration.num_hours()*60), 
                        player.duration.num_seconds()-(player.duration.num_minutes()*60))
                ]
        );
        }
        
        self.server_info = Some(server_info);
        self.items = players_rows;        
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }    
}

fn setup_panic() {
    let raw_handle = std::io::stdout().into_raw_mode().unwrap();
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        raw_handle
            .suspend_raw_mode()
            .unwrap();
        default_hook(info);
        // or better_panic::Settings::new().create_panic_handler()(info);
    }));
}

fn main() -> Result<(), Box<dyn Error>> {

    setup_panic();
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;    
    let stdout = AlternateScreen::from(stdout.into_raw_mode().unwrap());
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.clear().unwrap();

    let events = Events::new();

    let mut table = StatefulTable::new();

    // Input
    loop {
        terminal.draw(|f| {
            let rects = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .margin(5)
                .split(f.size());

            let selected_style = Style::default()
                .fg(Color::Blue);
                
            let normal_style = Style::default()
                .fg(Color::White);

            let header = ["Name", "Score", "Duration"];
            
            let rows = table
                .items
                .iter()
                .map(|i| Row::StyledData(i.iter(), normal_style));
            
            let server_name = match &table.server_info {
                Some(server_info) => format!("{} | Map: {} | Players: {}/{}", &server_info.name, &server_info.map, &server_info.players, &server_info.max_players),
                _ => "Nothing to see here".to_string()
            };
                
            let t = Table::new(header.iter(), rows)
                .block(Block::default().borders(Borders::ALL).title(server_name))
                .highlight_style(selected_style)
                .highlight_symbol(">> ")
                .widths(&[
                    Constraint::Percentage(50),
                    Constraint::Length(30),
                    Constraint::Max(15),
                ]);

            // let t = Table::new(["label", "value"], rows: R)
            f.render_stateful_widget(t, rects[0], &mut table.state);
        }).expect("something went wrong");

        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Down => {
                    table.next();
                }
                Key::Up => {
                    table.previous();
                }
                _ => {}
            },
            Event::Tick => {
                table.update(); 
            }
        }        
    }

    Ok(())
}

// fn main() {
//     let mut server = Server::connect("178.236.67.44:27015");
//     match server.get_server_info() {
//         Response::Error(err) => panic!("Failed to fetch server info. {}", err),
//         Response::Ok(server_info) => {
//             println!("{:#?}", server_info);

//             match server.get_players() {
//                 Response::Error(err) => panic!("Failed to fetch players. {}", err),
//                 Response::Ok(player_response) => {
//                     println!("{:#?}", player_response);
//                 }
//             }
//         }
//     }
// }
