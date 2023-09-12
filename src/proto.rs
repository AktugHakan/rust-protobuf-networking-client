use std::io::{Error, ErrorKind, Read, Write};

use prost::Message;

use crate::protocom::{
    request::{self, request::RequestType, BtnInterrupt, File, FileAccept, Info, LedControl},
    response::{self, response::ResponseType},
};
pub enum Command {
    Led(bool),
    Info,
    BtnInterrupt(u32),
    File(String),
    FileAccept(bool),
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
        _ => return Err(Error::new(ErrorKind::Unsupported, "Shouldn't be here???")),
    }
    let encoded_size = msg.encoded_len();

    let mut encoded_message: Vec<u8> = Vec::with_capacity(encoded_size);
    msg.encode(&mut encoded_message)?;

    return Ok(encoded_message);
}

pub fn decode_response_or_panic(message: &[u8]) -> response::Response {
    response::Response::decode(message).unwrap()
}

fn accept_file(connection: &mut std::net::TcpStream) {
    let file_accept_request = generate_message(&Command::FileAccept(true)).unwrap();
    connection
        .write(&file_accept_request)
        .expect("Couldn't send file acceptance");
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
            if file_header.status.unwrap() {
                let file_name = file_header.name.unwrap();
                println!(
                    "Downloading {} ({}) bytes",
                    file_name,
                    file_header.size.unwrap()
                );

                // Send file_accept request to server to start transfer
                accept_file(connection);

                // Read file
                let mut pb_file_buffer: Vec<u8> =
                    vec![0; usize::try_from(file_header.size.unwrap()).unwrap() + 100];
                let resp_len = connection.read(&mut pb_file_buffer).unwrap();

                let decoded_file = response::Response::decode(&pb_file_buffer[..resp_len]).unwrap();

                if let ResponseType::File(file) = decoded_file.response_type.unwrap() {
                    let recieved_files_folder = std::path::Path::new("recieved_files/");
                    let mut new_file: std::fs::File =
                        std::fs::File::create(recieved_files_folder.join(file_name.clone()))
                            .expect("Couldn't create file.");
                    new_file
                        .write(&file.file.unwrap())
                        .expect("Couldn't write to file");
                } else {
                    panic!("Expected a file but got another message type.");
                }
                println!("Downloaded {} successfully", file_name);
            } else {
                println!("File not found!");
            }
        }
        ResponseType::File(_) => {
            panic!("File arrived without a header!");
        }
    }
}
