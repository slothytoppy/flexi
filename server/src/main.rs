use std::ffi::CStr;
use std::os::fd::OwnedFd;
use std::{
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    process::Command,
};

use nix::unistd::Pid;

#[derive(Debug)]
/// multiplexer is the servers file descriptor to use, the "terminal" is the pty that acts as a
/// shell for executing commands
pub struct Pty {
    multiplexer: OwnedFd,
    child: Pid,
}

impl Pty {
    fn new() -> Self {
        unsafe {
            match nix::pty::forkpty(None, None).unwrap() {
                nix::pty::ForkptyResult::Child => match option_env!("SHELL") {
                    Some(shell) => {
                        let shell = format!("{shell}\0");
                        _ = nix::unistd::execv(
                            CStr::from_bytes_with_nul(shell.as_bytes()).unwrap(),
                            &[CStr::from_bytes_with_nul(shell.as_bytes()).unwrap()],
                        );
                        unreachable!()
                    }
                    None => {
                        _ = nix::unistd::execv(
                            CStr::from_bytes_until_nul(b"/bin/sh\0").unwrap(),
                            &[CStr::from_bytes_until_nul(b"/bin/sh\0").unwrap()],
                        );
                        unreachable!()
                    }
                },

                nix::pty::ForkptyResult::Parent { child, master } => Self {
                    multiplexer: master,
                    child,
                },
            }
        }
    }

    fn write(&mut self, val: &str) {
        let _ = nix::unistd::write(&self.multiplexer, val.as_bytes());
    }

    fn read(&self, buf: &mut [u8]) -> Result<usize, nix::Error> {
        nix::unistd::read(&self.multiplexer, buf)
    }
}

#[derive(Debug)]
pub struct Pane {
    default_cwd: Option<PathBuf>,
    last_run_cmd: Option<Command>,
    pty: Pty,
}

impl Pane {
    fn new() -> Self {
        let mut pty = Pty::new();
        pty.write("\x1b[2J\x1b[H");

        Self {
            default_cwd: None,
            last_run_cmd: None,
            pty,
        }
    }
}

#[derive(Debug)]
pub struct Session {
    default_cwd: Option<PathBuf>,
    panes: Vec<Pane>,
    selected_pane: usize,
}

impl Session {
    fn new() -> Self {
        Self {
            default_cwd: None,
            panes: vec![Pane::new()],
            selected_pane: 0,
        }
    }

    fn push(&mut self, pane: Pane) {
        self.panes.push(pane);
    }

    fn remove(&mut self, idx: usize) {
        self.panes.remove(idx);
    }
}

#[derive(Debug)]
pub struct Server {
    sessions: Vec<Session>,
    selected_session: usize,
}

impl Server {
    fn new() -> Self {
        Self {
            sessions: vec![Session::new()],
            selected_session: 0,
        }
    }

    fn listen(&mut self) {
        let conn = TcpListener::bind("127.0.0.1:8000").unwrap();
        let mut buff = [0; 1024];
        let mut clients = vec![];
        _ = conn.set_nonblocking(true);
        let session = self.sessions.get_mut(self.selected_session).unwrap();

        loop {
            match conn.accept() {
                Ok((mut stream, _)) => {
                    _ = stream.write(b"\x1b[2J\x1b[H");
                    clients.push(stream);
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock {
                        panic!("{e:?}")
                    }
                }
            }

            for stream in clients.iter_mut() {
                let r = stream.read(&mut buff).unwrap();
                if r == 0 {
                    continue;
                }
                let pane = session.panes.get_mut(session.selected_pane).unwrap();
                let input = str::from_utf8(&buff[0..r]).unwrap();
                pane.pty.write(input);
                let n = pane.pty.read(&mut buff).unwrap();
                if n == 0 {
                    continue;
                }
                _ = std::io::stdout().flush();
                _ = stream.write(&buff[0..n]).unwrap();
            }
        }
    }
}

#[derive(Debug)]
pub enum ServerRequest {
    Connect(String),
    CreateSession(String),
    ListSessions,
}

fn main() {
    Server::new().listen();
}
