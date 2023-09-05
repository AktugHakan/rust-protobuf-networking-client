use std::io::{self, Error, ErrorKind, Write};

use crate::proto::Command;

pub fn read_command() -> Result<Command, Error> {
    print!("_> ");
    io::stdout().flush().unwrap();

    let mut input_buffer = String::new();
    io::stdin().read_line(&mut input_buffer)?;

    let args: Vec<&str> = input_buffer.split(' ').collect();
    match args.get(0).unwrap_or(&"  ").trim() {
        "led" => match args.get(1).unwrap_or(&"  ").trim() {
            "on" => return Ok(Command::Led(true)),
            "off" => return Ok(Command::Led(false)),
            _ => return Err(Error::new(ErrorKind::InvalidInput, "Unknown LED state")),
        },
        "info" => return Ok(Command::Info),
        "button" => {
            return Ok(Command::BtnInterrupt(
                args.get(1).unwrap_or(&"0").trim().parse::<u32>().unwrap(),
            ))
        }
        "file" => return Ok(Command::File(args.get(1).unwrap_or(&" ").to_string())),
        "exit" => return Ok(Command::Exit),
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Unknown command")),
    }
}

pub fn connection_info_from_args() -> String {
    let mut cmdline_args = std::env::args();
    if cmdline_args.len() != 3 {
        panic!("Usage: client <ip> <port>");
    }

    cmdline_args.next();
    let ip_addr = cmdline_args.next().unwrap();
    let port = cmdline_args.next().unwrap();

    ip_addr + ":" + &port
}
