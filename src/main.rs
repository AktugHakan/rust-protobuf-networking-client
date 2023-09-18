use client::cmd_io;
use client::proto;
use client::proto::Command;
use std::net::TcpStream;

fn main() {
    // Connect to server
    let connection_info = cmd_io::connection_info_from_args();
    let mut socket = TcpStream::connect(connection_info).unwrap();

    loop {
        // Get command from user over command-line
        let command = read_command_until_valid();

        if let Command::Exit = command {
            break; // if message is exit; break the main loop
        }

        let resp = proto::send_request(&mut socket, &command);

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
