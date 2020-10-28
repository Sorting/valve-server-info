use crate::bytereader::ByteReader;
use crate::constants;
use chrono::Duration;
use std::net::UdpSocket;

pub enum Response<T> {
    Ok(T),
    Error(String),
}

pub struct Server {
    socket: UdpSocket    
}

#[derive(Debug)]
pub struct ServerInfo {
    pub header: u8,
    pub protocol: u8,
    pub name: String,
    pub map: String,
    pub folder: String,
    pub game: String,
    pub id: i16,    
    pub players: u8,
    pub max_players: u8,
    pub bots: u8,
    pub server_type: ServerType,
    pub environment: Environment,
    pub server_visibility: ServerVisibility,
    pub vac: bool,
    pub ship_mode: Option<ShipMode>,
    pub witnesses: Option<u8>,
    pub duration: Option<chrono::Duration>,
    pub version: String,
    pub edf: Option<u8>,
    pub port: Option<i16>,
    pub steam_id: Option<i32>,
    pub source_tv_port: Option<i16>,
    pub source_tv_name: Option<String>,
    pub keywords: Option<String>,
    pub game_id: Option<i16>,
}

#[derive(Debug)]
pub struct Player {
    pub index: u8,
    pub name: String,
    pub score: u32,
    pub duration: chrono::Duration,
    pub deaths: u32,
    pub money: Option<u32>,
}

#[derive(Debug)]
pub struct PlayersResponse {
    pub header: u8,
    pub players: Vec<Player>,
    pub is_ship: bool,
}

#[derive(Debug)]
pub enum ServerType {
    Dedicated,
    NonDedicated,
    SourceTvRelay,
    Unkown,
}

impl ServerType {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x00    => Self::Dedicated,
            0x64    => Self::Dedicated,
            0x6C    => Self::NonDedicated,
            0x70    => Self::SourceTvRelay,        
            _       => Self::Unkown,
        }
    }
}

#[derive(Debug)]
pub enum ShipMode {
    Hunt,
    Elimination,
    Duel,
    Deathmatch,
    VipTeam,
    TeamElimination,
    Unknown,
}

impl ShipMode {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => Self::Hunt,
            0x01 => Self::Elimination,
            0x02 => Self::Duel,
            0x03 => Self::Deathmatch,
            0x04 => Self::VipTeam,
            0x05 => Self::TeamElimination,
            _    => Self::Unknown,
        }
    }
}

#[derive(Debug)]
pub enum Environment {
    Linux,
    Windows,
    Mac,
    Unknown,
}

impl Environment {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x6C        => Self::Linux,
            0x77        => Self::Windows,
            0x6D | 0x6F => Self::Mac,
            _           => Self::Unknown,            
        }
    }
}

#[derive(Debug)]
pub enum ServerVisibility {
    Public,
    Private,
    Unknown,
}

impl ServerVisibility {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => Self::Public,
            0x01 => Self::Private,
            _    => Self::Unknown,
        }
    }
}

impl Server {
    pub fn connect(ip: &str) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:8899")
            .expect(&format!("Failed to connect to {}", ip)[..]);            
        let timout_duration = chrono::Duration::seconds(5)
            .to_std()
            .expect("something went wrong");

        socket.set_write_timeout(Some(timout_duration)).expect("Failed to set write timeout");
        socket.set_read_timeout(Some(timout_duration)).expect("Failed to set read timeout");

        socket.connect(ip).expect("");

        Self {
            socket
        }
    }

    pub fn send(&mut self, request: &[u8]) -> Response<ByteReader> {
        self.socket
            .send(request)
            .expect("Failed to send the request");

            let buf = &mut [0x00; 1400].to_vec();
            match self.socket.recv(buf) {
                Ok(_) => {
                    let mut reader = ByteReader::new(buf.to_vec());
                    let header_response = reader.get_long();
                    
                    if header_response == constants::SIMPLE_RESPONSE_HEADER {
                        Response::Ok(ByteReader::new(reader.peek_remaining_bytes().to_vec()))
                    } else {
                        Response::Error(format!("Unexpected header received from the server: {:?}", header_response))
                    }                
                },
                Err(err) => Response::Error(format!("Failed to read bytes, error: {}", err))
            }
    }

    pub fn get_server_info(&mut self) -> Response<ServerInfo> {
        match self.send(&constants::SERVER_INFO_REQUEST) {
            Response::Error(reason) => Response::Error(format!("Failed to get server info, reason: {}", reason)),
            Response::Ok(mut buf) => {
                let server_info = ServerInfo { 
                    header: buf.get_byte(),
                    protocol: buf.get_byte(),
                    name: buf.get_string(),
                    map: buf.get_string(),
                    folder: buf.get_string(),
                    game: buf.get_string(),
                    id: buf.get_short(),
                    players: buf.get_byte(),
                    max_players: buf.get_byte(),
                    bots: buf.get_byte(),
                    server_type: ServerType::from_byte(buf.get_byte()),
                    environment: Environment::from_byte(buf.get_byte()),
                    server_visibility: ServerVisibility::from_byte(buf.get_byte()),
                    vac: buf.get_byte() == 0x01,
                    ship_mode: None,
                    witnesses: None,
                    duration: None,
                    version: "".to_string(),
                    edf: None,
                    port: None,
                    steam_id: None,
                    source_tv_port: None,
                    source_tv_name: None,                    
                    game_id: None,
                    keywords: None,

                };
                if server_info.id == 2400 {
                    Response::Ok(ServerInfo { 
                        ship_mode: Some(ShipMode::from_byte(buf.get_byte())),
                        witnesses: Some(buf.get_byte()),
                        duration: Some(Duration::seconds(buf.get_byte() as i64)),
                        version: buf.get_string(),
                        .. server_info 
                    })
                } else {                   
                    Response::Ok(ServerInfo { version: buf.get_string(), .. server_info })
                }
            }
        }        
    }
    pub fn get_players(&mut self) -> Response<PlayersResponse> {
        let mut request = constants::PLAYERS_CHALLANGE_RESPONSE.clone();

        match self.send(&request) {
            Response::Error(reason) => Response::Error(format!("Failed to send inital player challenge request, reason: {}", reason)),
            Response::Ok(mut buf) => {
                buf.get_byte(); // ignore header
                
                request[5] = buf.get_byte();
                request[6] = buf.get_byte();
                request[7] = buf.get_byte();
                request[8] = buf.get_byte();

                match self.send(&request) {
                    Response::Error(reason) => Response::Error(format!("Failed to send second player challenge request, reason: {}", reason)),
                    Response::Ok(mut buf) => {
                        let header = buf.get_byte();
                        let player_count = buf.get_byte();
                        let mut players = vec![];

                        for _ in 0..player_count {
                            players.push(Player {
                                index: buf.get_byte(),
                                name: buf.get_string(),
                                score: buf.get_long(),                                
                                deaths: 0,
                                duration: Duration::seconds(buf.get_float() as i64),                                
                                money: None,
                            });
                        }

                        Response::Ok(PlayersResponse {
                            header,
                            is_ship: false,
                            players,
                        })
                    }
                }                
            },
        }
    }
}