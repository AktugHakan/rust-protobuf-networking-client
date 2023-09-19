use std::{
    collections::BinaryHeap,
    ffi::OsStr,
    io::{Result, Write},
    net::TcpStream,
    path::PathBuf,
};

use crate::{
    network::TcpWithSize,
    proto::{self, Command},
    protocom::response::{self, FileHeader},
};

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

pub fn file_download_routine(socket: &mut std::net::TcpStream, file_header: FileHeader) {
    if file_header.status.unwrap() {
        let segment_count = file_header.segment_count();
        let file_name = file_header.name.unwrap();
        println!(
            "Downloading {} ({}) bytes",
            file_name,
            file_header.size.unwrap()
        );

        // Send file_accept request to server to start transfer
        crate::file_op::start_server_file_mode(socket);
        println!("Accepted file");

        crate::file_op::download_file(
            socket,
            file_name.as_str(),
            &"recieved_files/",
            segment_count,
        );

        exit_server_file_mode(socket);

        println!("Downloaded {} successfully", file_name);
    } else {
        println!("File not found!");
    }
}

fn download_file<T: Into<PathBuf> + AsRef<OsStr>>(
    connection: &mut std::net::TcpStream,
    file_name: &str,
    download_directory: &T,
    segment_count: u64,
) {
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

    save_file(download_directory, file_name, segments).expect("File cannot be saved on disk.");
}

fn recieve_file_segment(socket: &mut TcpStream, segment_no: u64) -> Vec<u8> {
    proto::only_send_request(socket, &Command::FileAck(Some(segment_no)));
    if let response::response::ResponseType::File(file) =
        proto::recieve_response(socket).response_type.unwrap()
    {
        return file.file.unwrap();
    } else {
        panic!("Expected a file but got another response.");
    }
}

fn start_server_file_mode(connection: &mut std::net::TcpStream) {
    let file_accept_request = proto::generate_message(&Command::FileAccept(true)).unwrap();
    connection
        .send(&file_accept_request)
        .expect("Cannot accept file.");
}

fn save_file<T>(directory: &T, file_name: &str, mut segments: BinaryHeap<FileSegment>) -> Result<()>
where
    T: Into<std::path::PathBuf> + AsRef<OsStr>,
{
    let mut save_dir: PathBuf = directory.into();
    std::fs::create_dir_all(save_dir.clone())?;

    save_dir.push(file_name);
    let mut file = std::fs::File::create(save_dir)?;
    while !segments.is_empty() {
        let data = segments.pop().unwrap().data;
        file.write(&data)?;
    }
    Ok(())
}

fn exit_server_file_mode(socket: &mut TcpStream) {
    proto::only_send_request(socket, &Command::FileAck(None));
}
