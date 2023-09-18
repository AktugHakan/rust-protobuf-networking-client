use std::{
    collections::BinaryHeap,
    io::{Error, ErrorKind, Write},
    net::TcpStream,
};

use prost::Message;

use crate::{
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
    FileAck(u64),
    Exit,
}

#[derive(Eq)]
struct FileSegment {
    order: u64,
    data: Vec<u8>,
}

impl Ord for FileSegment {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.order.cmp(&self.order)
    }
}

impl PartialOrd for FileSegment {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for FileSegment {
    fn eq(&self, other: &Self) -> bool {
        self.order == other.order
    }
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
                .insert(RequestType::FileAck(FileAck { next: Some(*next) }));
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

fn accept_file(connection: &mut std::net::TcpStream) {
    let file_accept_request = generate_message(&Command::FileAccept(true)).unwrap();
    connection
        .send(&file_accept_request)
        .expect("Cannot accept file.");
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
                let segment_count = file_header.segment_count();
                let file_name = file_header.name.unwrap();
                println!(
                    "Downloading {} ({}) bytes",
                    file_name,
                    file_header.size.unwrap()
                );

                // Send file_accept request to server to start transfer
                accept_file(connection);
                println!("Accepted file");

                download_file(connection, file_name.as_str(), segment_count);

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

fn download_file(
    connection: &mut std::net::TcpStream,
    file_name: &str,
    segment_count: u64,
) -> std::fs::File {
    let mut segments: BinaryHeap<FileSegment> = BinaryHeap::with_capacity(segment_count as usize);

    for i in 0..segment_count {
        let data = recieve_file_segment(connection, i);
        let file_segment = FileSegment {
            data: data,
            order: i,
        };
        segments.push(file_segment);
    }

    // TODO!!!
    // Check if all segments exist ....

    let recieved_files_folder = std::path::Path::new("recieved_files/");
    let mut file = std::fs::File::create(recieved_files_folder.join(file_name)).unwrap();
    while !segments.is_empty() {
        let data = segments.pop().unwrap().data;
        file.write(&data).expect("Cannot write to file");
    }

    file
}

pub fn recieve_response(socket: &mut TcpStream) -> Response {
    let resp = socket.recieve().unwrap();
    decode_response_or_panic(&resp)
}

pub fn recieve_file_segment(socket: &mut TcpStream, segment_no: u64) -> Vec<u8> {
    only_send_request(socket, &Command::FileAck(segment_no));
    if let response::response::ResponseType::File(file) =
        recieve_response(socket).response_type.unwrap()
    {
        return file.file.unwrap();
    } else {
        panic!("Expected a file but got another response.");
    }
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
