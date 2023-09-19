use std::{
    io::{Error, ErrorKind},
    net::TcpStream,
};

use prost::Message;

use crate::{
    file_op,
    network::TcpWithSize,
    protocom::{
        request::{
            self, request::RequestType, BtnInterrupt, File, FileAccept, FileAck, Info, LedControl,
        },
        response::{self, response::ResponseType, Response},
    },
};
pub enum Command {
    Led(bool),
    Info,
    BtnInterrupt(u32),
    File(String),
    FileAccept(bool),
    FileAck(Option<u64>),
    Exit,
}

pub fn generate_message(command: &Command) -> Result<Vec<u8>, Error> {
    let mut msg = request::Request::default();
    match command {
        Command::Led(enable) => {
            let _ = msg.request_type.insert(RequestType::Ledctrl(LedControl {
                enable: Some(*enable),
            }));
        }
        Command::Info => {
            let _ = msg.request_type.insert(RequestType::Info(Info {}));
        }
        Command::BtnInterrupt(timeout_us) => {
            let _ = msg.request_type.insert(RequestType::Btnint(BtnInterrupt {
                timeout_us: Some(*timeout_us),
            }));
        }
        Command::File(filename) => {
            let _ = msg.request_type.insert(RequestType::File(File {
                file_name: Some(filename.clone()),
            }));
        }
        Command::FileAccept(accept) => {
            let _ = msg.request_type.insert(RequestType::FileAccept(FileAccept {
                accept: Some(*accept),
            }));
        }
        Command::FileAck(next) => {
            let _ = msg
                .request_type
                .insert(RequestType::FileAck(FileAck { next: *next }));
        }
        _ => return Err(Error::new(ErrorKind::Unsupported, "Shouldn't be here???")),
    }
    let encoded_size = msg.encoded_len();

    let mut encoded_message: Vec<u8> = Vec::with_capacity(encoded_size);
    msg.encode(&mut encoded_message)?;

    return Ok(encoded_message);
}

pub fn decode_response_or_panic(message: &[u8]) -> response::Response {
    println!("Recieved length: {}", message.len());
    response::Response::decode(message).unwrap()
}

pub fn response_action(resp: response::Response, connection: &mut std::net::TcpStream) {
    match resp.response_type.unwrap() {
        ResponseType::Status(led_status) => {
            if led_status.status.unwrap() {
                println!("LED change successful.");
            } else {
                println!("LED change failed.");
            }
        }
        ResponseType::ServerInfo(info) => {
            println!("IP:{} | Port:{}", info.ip.unwrap(), info.port.unwrap());
        }
        ResponseType::FileHeader(file_header) => {
            file_op::file_download_routine(connection, file_header)
        }
        ResponseType::File(_) => {
            panic!("File arrived without a header!");
        }
    }
}

pub fn recieve_response(socket: &mut TcpStream) -> Response {
    let resp = socket.recieve().unwrap();
    decode_response_or_panic(&resp)
}

pub fn send_request(socket: &mut TcpStream, command: &Command) -> Response {
    let encoded_message = generate_message(command).expect("Couldn't generate message");
    socket
        .send(&encoded_message)
        .expect("Couldn't send message.");
    recieve_response(socket)
}

pub fn only_send_request(socket: &mut TcpStream, command: &Command) {
    let encoded_message = generate_message(command).expect("Couldn't generate message");
    socket
        .send(&encoded_message)
        .expect("Couldn't send message.")
}
