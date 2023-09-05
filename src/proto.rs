use std::io::{Error, ErrorKind};

use prost::Message;

use crate::protocom::{
    request::{self, request::RequestType, BtnInterrupt, File, Info, LedControl},
    response::{self, response::ResponseType},
};

pub enum Command {
    Led(bool),
    Info,
    BtnInterrupt(u32),
    File(String),
    Exit,
}

pub enum Response {
    Status,
    Info(String),
    FileHeader(String, u64, bool),
    File(), // TODO: type of byte array
}

pub fn generate_message(command: &Command) -> Result<Vec<u8>, Error> {
    let mut msg = request::Request::default();
    match command {
        Command::Led(enable) => {
            let _ = msg
                .request_type
                .insert(RequestType::Ledctrl(LedControl { enable: *enable }));
        }
        Command::Info => {
            let _ = msg.request_type.insert(RequestType::Info(Info {}));
        }
        Command::BtnInterrupt(timeout_us) => {
            let _ = msg.request_type.insert(RequestType::Btnint(BtnInterrupt {
                timeout_us: *timeout_us,
            }));
        }
        Command::File(filename) => {
            let _ = msg.request_type.insert(RequestType::File(File {
                file_name: filename.clone(),
            }));
        }

        _ => return Err(Error::new(ErrorKind::Unsupported, "Shouldn't be here???")),
    }
    let encoded_size = msg.encoded_len();
    println!("### generate_message(): msgLen: {}", encoded_size);

    let mut encoded_message: Vec<u8> = Vec::with_capacity(encoded_size);
    msg.encode(&mut encoded_message)?;

    return Ok(encoded_message);
}

pub fn decode_response_or_panic(message: Vec<u8>) -> response::Response {
    response::Response::decode(message.as_slice()).unwrap()
}

pub fn response_action(resp: response::Response) {
    match resp.response_type.unwrap() {
        ResponseType::Status(led_status) => {
            if led_status.status {
                println!("LED change successful.");
            } else {
                println!("LED change failed.");
            }
        }
        ResponseType::ServerInfo(info) => {
            println!("IP:{} | Port:{}", info.ip, info.port);
        }
        ResponseType::FileHeader(file_header) => {
            // File download routine.
        }
        ResponseType::File(_) => {
            panic!("File arrived without a header!");
        }
    }
}
