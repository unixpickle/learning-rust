// There's no guarantee that destructors will be called,
// so it's possible to leak MutexGuard locks without an
// unsafe block.

use std::io;
use std::io::BufRead;
use std::mem;

fn main() {
    // If you comment out this line, the program works.
    mess_up_stdin();

    try_to_use_stdin();
}

fn mess_up_stdin() {
    let stdin = io::stdin();
    let lock = stdin.lock();
    mem::forget(lock);
}

fn try_to_use_stdin() {
    let stdin = io::stdin();
    println!("attempting to lock stdin...");
    let mut lock = stdin.lock();
    let mut line = String::new();
    println!("reading a line...");
    match lock.read_line(&mut line) {
        Ok(_) => {
            print!("{}", line);
        }
        Err(error) => {
            println!("error: {}", error);
        }
    }
}
