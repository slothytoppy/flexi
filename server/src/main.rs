use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};

use messages::ServerRequest;
use pty::Pty;
use tracing::info;

mod pty;

#[derive(Debug)]
pub struct Pane {
    pty: Pty,
}

impl Pane {
    fn new() -> Self {
        let pty = Pty::new();

        Self { pty }
    }
}

#[derive(Debug)]
pub struct Session {
    panes: Vec<Pane>,
    selected_pane: usize,
}

impl Session {
    fn new() -> Self {
        Self {
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
    sessions: Session,
    listener: UnixListener,
    client: UnixStream,
}

impl Server {
    fn new() -> Self {
        if std::fs::exists(messages::CONNECTION_PATH).unwrap() {
            _ = std::fs::remove_file(messages::CONNECTION_PATH);
        }
        let listener = UnixListener::bind(messages::CONNECTION_PATH).unwrap();

        let (client, _addr) = listener.accept().unwrap();

        let mut server = Self {
            sessions: Session::new(),
            listener,
            client,
        };

        server.clear_tty();

        server
    }

    fn clear_tty(&mut self) {
        let mut buff = [0; 1024];
        let pane = self
            .sessions
            .panes
            .get_mut(self.sessions.selected_pane)
            .unwrap();
        _ = self.client.write(b"\x1b[2J\x1b[H");
        _ = pane.pty.write(b"");
        _ = pane.pty.flush();
        let n = pane.pty.read(&mut buff).unwrap();
        _ = self.client.write(&buff[0..n]);
    }

    fn new_pane(&mut self) {
        let mut pane = Pane::new();
        self.clear_tty();

        self.sessions.panes.push(pane);
    }

    fn read_request(&mut self) -> Result<ServerRequest, messages::RequestError> {
        let mut buff = [0; 1024];
        _ = self.client.read(&mut buff).unwrap();
        info!("request = {:?}", buff[0]);
        ServerRequest::try_from(buff[0])
    }

    fn run(&mut self) {
        loop {
            if let Ok(req) = self.read_request() {
                info!(?req);
                match req {
                    ServerRequest::Connect => todo!(),
                    ServerRequest::Disconnect => {
                        println!("closing client");
                        _ = self.client.shutdown(std::net::Shutdown::Both);
                    }
                    ServerRequest::NewPane => todo!(),
                }
            }
        }
    }
}

fn main() {
    Server::new().run();
}
