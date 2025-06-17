use std::{
    io::{stdin, stdout, Read, Write},
    net::TcpStream,
    os::fd::AsFd,
};

use nix::poll::{poll, PollFlags, PollTimeout};

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8000").unwrap();
    let mut buf = [0; 1024];
    let r = stream.read(&mut buf).unwrap();
    _ = stdout().write(&buf[0..r]);
    print!("{}", str::from_utf8(&buf[0..r]).unwrap());
    _ = std::io::stdout().flush();

    let stdin = stdin();
    let mut fds = [nix::poll::PollFd::new(stdin.as_fd(), PollFlags::POLLIN)];
    // _ = crossterm::terminal::enable_raw_mode();

    loop {
        if poll(&mut fds, PollTimeout::MAX).unwrap() > 0 {
            let n = std::io::stdin().read(&mut buf).unwrap();
            let _ = stream.write(&buf[0..n]);
            match stream.read(&mut buf) {
                Ok(r) => {
                    let s = unsafe { str::from_utf8_unchecked(&buf[0..r]) };
                    print!("{s}");
                    _ = std::io::stdout().flush();
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock {
                        panic!("{e:?}")
                    }
                }
            }
        }
    }
}
