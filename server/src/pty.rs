use std::{
    ffi::CString,
    io::{Read, Write},
    os::fd::{AsFd, AsRawFd, OwnedFd},
};

use nix::unistd::Pid;

#[derive(Debug)]
/// multiplexer is the servers file descriptor to use, the "terminal" is the pty that acts as a
/// shell for executing commands
#[allow(unused)]
pub struct Pty {
    multiplexer: OwnedFd,
    child: Pid,
}

impl Pty {
    pub fn new() -> Self {
        let mut winsize = nix::pty::Winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        unsafe {
            let ret = nix::libc::ioctl(
                std::io::stdin().as_raw_fd(),
                nix::libc::TIOCGWINSZ,
                &mut winsize,
            );
            assert_eq!(ret, 0);

            match nix::pty::forkpty(Some(&winsize), None).unwrap() {
                nix::pty::ForkptyResult::Child => match option_env!("SHELL") {
                    Some(shell) => {
                        let shell = CString::new(shell).unwrap();

                        _ = nix::unistd::execv(&shell, &[&shell]);
                        unreachable!()
                    }
                    None => {
                        let shell = CString::new("/bin/sh").unwrap();
                        _ = nix::unistd::execv(&shell, &[&shell]);
                        unreachable!()
                    }
                },

                nix::pty::ForkptyResult::Parent { child, master } => {
                    nix::libc::ioctl(master.as_raw_fd(), nix::libc::TIOCSWINSZ, &winsize);
                    Self {
                        multiplexer: master,
                        child,
                    }
                }
            }
        }
    }
}

impl Write for Pty {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match nix::unistd::write(&self.multiplexer, buf) {
            Ok(n) => Ok(n),
            Err(e) => Err(std::io::Error::from(e)),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let res = unsafe { nix::libc::fdatasync(self.multiplexer.as_raw_fd()) };
        Err(std::io::Error::from_raw_os_error(res))
    }
}

impl Read for Pty {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match nix::unistd::read(self.multiplexer.as_fd(), buf) {
            Ok(n) => Ok(n),
            Err(e) => Err(std::io::Error::from(e)),
        }
    }
}
