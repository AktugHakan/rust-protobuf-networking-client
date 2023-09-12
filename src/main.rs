use client::cmd_io;
use client::proto;
use client::proto::Command;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;

fn main() {
    let connection_info = cmd_io::connection_info_from_args();
    let mut socket = TcpStream::connect(connection_info).unwrap();

    loop {
        let command = read_command_until_valid();

        if let Command::Exit = command {
            break; // if message is exit; break the main loop
        }

        let encoded_message = encode_or_panic(&command);

        // Send message and wait for answer.
        socket.write(&encoded_message).unwrap();
        let mut resp: Vec<u8> = vec![0; 1024];
        let resp_len = socket
            .read(&mut resp)
            .expect("Cannot recieve server response");
        let resp = proto::decode_response_or_panic(&resp[..resp_len]);
        proto::response_action(resp, &mut socket);
    }
}

// Read user entry until its valid.
fn read_command_until_valid() -> Command {
    loop {
        let selected_command = cmd_io::read_command();
        if let Ok(command) = selected_command {
            return command;
        } else if let Err(error) = selected_command {
            println!("{}", error);
        }
    }
}

// Get encoded message or panic
fn encode_or_panic(command: &Command) -> Vec<u8> {
    proto::generate_message(command).expect("Cannot write to byte buffer after encoding.")
}
