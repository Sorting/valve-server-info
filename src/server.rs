use crate::bytereader::ByteReader;
use crate::constants;
use std::io::Result;
use std::time::Duration;
use std::net::UdpSocket;
use std::convert::{TryInto};

pub enum ServerResponse<T> {
    SuccessResponse(T),
    Failure(String),
}

pub enum Response {
    Ok(ByteReader),
    Failure(String),
}

pub struct Server {
    socket: UdpSocket,
    connected: bool,
}

pub struct ServerInfo {
    pub name: String,
    pub map: String,
    pub folder: String,
    pub game: String,
    pub id: i16,    
    pub players: u8,
    pub max_players: u8,
    pub bots: u8,
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

    pub fn send(&mut self, request: &[u8]) -> Response {
        println!("{:?}", request);
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

    pub fn get_server_info(&mut self) -> ServerResponse<ServerInfo> {
        match self.send(&constants::SERVER_INFO_REQUEST) {
            Response::Failure(reason) => ServerResponse::Failure(format!("Failed to get server infom reason: {}", reason)),
            Response::Ok(mut buf) => {
                buf.get_byte(); // header
                buf.get_byte(); //protocol                
                ServerResponse::SuccessResponse(
                    ServerInfo { 
                        name: buf.get_string(),
                        map: buf.get_string(),
                        folder: buf.get_string(),
                        game: buf.get_string(),
                        id: buf.get_short(),
                        players: buf.get_byte(),
                        max_players: buf.get_byte(),
                        bots: buf.get_byte(),
                    }
                )
            }
        }
    }
}