use std::process::{Command, exit};
use std::io::{Write, Read};
use std::fs::{File, remove_file};
use std::os::unix::io::{AsRawFd, RawFd};
use libc::{flock, LOCK_EX, LOCK_NB};

const FIFO_PATH: &str = "/tmp/my_fifo";

fn create_fifo() -> std::io::Result<()> {
    match std::fs::File::create(FIFO_PATH) {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

fn acquire_lock(fd: RawFd) -> std::io::Result<()> {
    let res = unsafe { flock(fd, LOCK_EX | LOCK_NB) };
    if res == -1 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn release_lock(fd: RawFd) -> std::io::Result<()> {
    let res = unsafe { flock(fd, libc::LOCK_UN) };
    if res == -1 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn parent_process() {
    let fifo = File::open(FIFO_PATH).expect("Unable to open FIFO for writing");
    let fd = fifo.as_raw_fd();
    
    acquire_lock(fd).expect("Failed to acquire lock");

    // Write data to the FIFO
    let mut fifo = File::create(FIFO_PATH).expect("Unable to open FIFO for writing");
    let message = "child process!";
    fifo.write_all(message.as_bytes()).expect("Error writing to FIFO");

    println!("Parent process sent message: {}", message);

    release_lock(fd).expect("Failed to release lock");

    let status = Command::new("echo")
        .arg("Parent process completed.")
        .status()
        .expect("Failed to wait for child process");
    
    if !status.success() {
        eprintln!("Error running echo command");
    }
}

fn child_process() {
    let fifo = File::open(FIFO_PATH).expect("Unable to open FIFO for reading");
    let fd = fifo.as_raw_fd();
    
    acquire_lock(fd).expect("Failed to acquire lock");

    let mut fifo = File::open(FIFO_PATH).expect("Unable to open FIFO for reading");

    let mut buffer = String::new();
    fifo.read_to_string(&mut buffer).expect("Error reading from FIFO");

    println!("Child process received message: {}", buffer);

    release_lock(fd).expect("Failed to release lock");

    remove_file(FIFO_PATH).expect("Unable to remove FIFO");
}

fn main() {
    create_fifo().expect("Unable to create FIFO");

    // Fork the process
    match unsafe { libc::fork() } {
        -1 => {
            eprintln!("Fork failed");
            exit(1);
        }
        0 => {
            // Child process
            child_process();
        }
        _ => {
            // Parent process
            parent_process();
        }
    }
}
