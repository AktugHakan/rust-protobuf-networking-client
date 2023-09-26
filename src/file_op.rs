use std::{
    collections::BinaryHeap,
    ffi::OsStr,
    io::{Read, Result, Seek, Write},
    net::TcpStream,
    path::PathBuf,
    vec,
};

use crate::{
    network::TcpWithSize,
    proto::{self, recieve_response, Command},
    protocom::{
        request::File,
        response::{self, response::ResponseType, FileHeader},
    },
};

use sha2::{Digest, Sha256};

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

        let mut file = crate::file_op::download_file(
            socket,
            file_name.as_str(),
            &"recieved_files/",
            segment_count,
        );

        let hash = exit_server_file_mode(socket);
        if check_file_integrity(&hash, &mut file) {
            println!("FILE OK");
        } else {
            println!("FILE CORRUPTED");
        }

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

    save_file(download_directory, file_name, segments).expect("File cannot be saved on disk.")
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

fn save_file<T>(
    directory: &T,
    file_name: &str,
    mut segments: BinaryHeap<FileSegment>,
) -> Result<std::fs::File>
where
    T: Into<std::path::PathBuf> + AsRef<OsStr>,
{
    let mut save_dir: PathBuf = directory.into();
    std::fs::create_dir_all(save_dir.clone())?;

    save_dir.push(file_name);
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(save_dir)?;
    while !segments.is_empty() {
        let data = segments.pop().unwrap().data;
        file.write(&data)?;
    }
    Ok(file)
}

fn check_file_integrity(sha2_digest: &[u8], file: &mut std::fs::File) -> bool {
    let mut hasher = Sha256::new();
    let mut file_buffer: Vec<u8> = Vec::with_capacity(file.metadata().unwrap().len() as usize);
    file.seek(std::io::SeekFrom::Start(0)).unwrap();
    file.read_to_end(&mut file_buffer).unwrap();
    hasher.update(file_buffer);
    let file_hash: Vec<u8> = hasher.finalize().to_vec();

    for (a, b) in sha2_digest.iter().zip(file_hash.iter()) {
        if *a != *b {
            return false;
        }
    }

    return true;
}

fn exit_server_file_mode(socket: &mut TcpStream) -> Vec<u8> {
    proto::only_send_request(socket, &Command::FileAck(None));
    let resp = recieve_response(socket);
    if let ResponseType::FileHash(hash) = resp.response_type.unwrap() {
        return hash.digest.unwrap();
    } else {
        panic!("Expected hash, recieved another type");
    }
}
