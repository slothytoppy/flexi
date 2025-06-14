extern crate nix;

use nix::{
    poll::{poll, PollFd, PollFlags, PollTimeout},
    pty::*,
};
use nix::{sys::wait::wait, unistd::*};
use std::ffi::CStr;
use std::io::{stdin, stdout, Read, Write};
use std::os::fd::{AsFd, AsRawFd};

fn main() {
    println!("starting flexi");

    match unsafe { nix::pty::forkpty(None, None) } {
        Ok(ForkptyResult::Child) => {
            let mut shell = match std::env::var("SHELL") {
                Ok(var) => var,
                Err(_) => "/bin/sh".to_string(),
            };
            shell.push('\0');
            let cstr = CStr::from_bytes_until_nul(shell.as_bytes()).unwrap();
            let _ = crossterm::terminal::enable_raw_mode();
            execvp(cstr, &[cstr]).expect("Failed to exec shell");
            unsafe { nix::libc::exit(0) };
        }

        Ok(ForkptyResult::Parent { master, .. }) => {
            // let _ = crossterm::terminal::enable_raw_mode();
            let stdin_fd = stdin();
            let stdin_fd = stdin_fd.as_fd();
            let master_input = dup(master.as_fd()).expect("Failed to dup master");
            let mut fds = [
                nix::poll::PollFd::new(stdin_fd, nix::poll::PollFlags::POLLIN),
                PollFd::new(master_input.as_fd(), PollFlags::POLLIN),
            ];

            loop {
                let mut output_buf = [0u8; 1024];
                match poll(&mut fds, PollTimeout::MAX) {
                    Ok(0) => continue,
                    Ok(_) => {
                        if fds[0].any().unwrap() {
                            let n = stdin().read(&mut output_buf).unwrap();
                            if output_buf[0] == b'q' {
                                println!("exiting flexi");
                                break;
                            }
                            write(&master, &output_buf[..n]).unwrap();
                        }

                        if fds[1].any().unwrap() {
                            let n = read(&master, &mut output_buf).unwrap();
                            {
                                stdout().lock().write_all(&output_buf[..n]).unwrap();
                            }
                            stdout().flush().unwrap();
                        }
                    }
                    Err(e) => {
                        panic!("{:?}", e)
                    }
                }
            }
            println!("broke from loop");
            let _ = close(master.as_raw_fd());
            let _ = wait(); // Wait for child to exit
        }

        Err(e) => {
            eprintln!("Fork failed: {}", e);
        }
    }
    println!("exited flexi");
}
