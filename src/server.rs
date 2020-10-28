use crate::bytereader::ByteReader;
use crate::constants;
use std::io::Result;
use std::time::Duration;
use std::net::UdpSocket;
use std::convert::{TryInto};

pub enum Response<T> {
    Ok(T),
    Failure(String),
}

pub struct Server {
    socket: UdpSocket,
    connected: bool,
}

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
pub enum Environment {
    Linux,
    Windows,
    Mac,
    Unknown,
}

impl Environment {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x6C            => Self::Linux,
            0x77            => Self::Windows,
            0x6D | 0x6F     => Self::Mac,
            _               => Self::Unknown,            
        }
    }
}

#[derive(Debug)]
pub enum ServerVisibility {
    Public,
    Private,
    Unknown
}

impl ServerVisibility {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x00    => Self::Public,
            0x01    => Self::Private,
            _       => Self::Unknown,
        }
    }
}

impl Server {
    pub fn bind(ip: &str) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:8899")
            .expect("Failed to connect to");            
        let timout_duration = Duration::from_secs(5);

        socket.set_write_timeout(Some(timout_duration)).expect("");
        socket.set_read_timeout(Some(timout_duration)).expect("");

        socket.connect(ip).expect("");

        Self {
            socket,
            connected: false
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
                        Response::Failure(format!("Unexpected header received from the server: {:?}", header_response))
                    }                
                },
                _ => Response::Failure(String::from("Failed to read bytes"))
            }
    }

    pub fn get_server_info(&mut self) -> Response<ServerInfo> {
        match self.send(&constants::SERVER_INFO_REQUEST) {
            Response::Failure(reason) => Response::Failure(format!("Failed to get server infom reason: {}", reason)),
            Response::Ok(mut buf) => {                         
                Response::Ok(
                    ServerInfo { 
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
                    }
                )
            }
        }
    }    
}