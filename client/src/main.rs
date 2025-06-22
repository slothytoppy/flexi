use std::{
    io::{stdin, stdout, ErrorKind, Read, Write},
    os::unix::net::UnixStream,
};

use crossterm::terminal::enable_raw_mode;
use messages::ClientMessage;

fn main() {
    let mut stream = UnixStream::connect(messages::CONNECTION_PATH).unwrap();
    let mut buf = [0; 1024];

    let r = stream.read(&mut buf).unwrap();
    _ = stdout().write(&buf[0..r]);
    _ = stdout().flush();

    // _ = enable_raw_mode();

    loop {
        match stdin().read(&mut buf) {
            Ok(n) => {
                if buf[0] as char == 'q' {
                    _ = stream.write(&[ClientMessage::Disconnect as u8]);
                    break;
                }
                let message = ClientMessage::try_from(buf[0]);
                if let Err(e) = &message {
                    println!("Error: {e:?}");
                }
                let message = message.unwrap();
                buf[0] = message as u8;

                let _ = stream.write(&buf[0..n]);
            }
            Err(e) => {
                if e.kind() != ErrorKind::WouldBlock {
                    panic!("{e:?}")
                }
            }
        };

        match stream.read(&mut buf) {
            Ok(r) => {
                _ = stdout().write(&buf[0..r]).unwrap();
                _ = stdout().flush();
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    panic!("{e:?}");
                }
            }
        };
    }
}
