// A Brainfuck (https://en.wikipedia.org/wiki/Brainfuck)
// interpreter written in Rust.

use std::fs::File;
use std::io::{Read, Write, stdin, stdout};
use std::iter::FromIterator;

const BUFFER_SIZE: usize = 8192;

fn main() {
    let code = read_program("program.bf");
    let mut data = Vec::<u8>::with_capacity(BUFFER_SIZE);
    for _ in 0..BUFFER_SIZE {
        data.push(0);
    }

    let mut code_ptr: usize = 0;
    let mut data_ptr: usize = 0;

    while code_ptr < code.len() {
        match code[code_ptr] {
            '>' => data_ptr += 1,
            '<' => data_ptr -= 1,
            '+' => data[data_ptr] = increment(data[data_ptr]),
            '-' => data[data_ptr] = decrement(data[data_ptr]),
            '.' => write_byte(data[data_ptr]),
            ',' => data[data_ptr] = read_byte(),
            '[' => {
                if data[data_ptr] == 0 {
                    code_ptr = matching_close(&code, code_ptr);
                }
            }
            ']' => {
                if data[data_ptr] != 0 {
                    code_ptr = matching_open(&code, code_ptr);
                }
            }
            _ => ()
        }
        code_ptr += 1;
    }
}

fn read_program(path: &str) -> Vec<char> {
    let mut res = String::new();
    let mut f = File::open(path).expect("Unable to open file.");
    f.read_to_string(&mut res).expect("Could not read program.");
    Vec::<char>::from_iter(res.chars())
}

fn increment(x: u8) -> u8 {
    if x == 0xff {
        0
    } else {
        x + 1
    }
}

fn decrement(x: u8) -> u8 {
    if x == 0 {
        0xff
    } else {
        x - 1
    }
}

fn write_byte(x: u8) {
    stdout().write(&[x]).expect("Could not write byte to stdout.");
}

fn read_byte() -> u8 {
    let mut res: [u8; 1] = [0];
    stdin().read_exact(&mut res).expect("Could not read byte from stdin.");
    res[0]
}

fn matching_close(code: &Vec<char>, code_ptr: usize) -> usize {
    let mut count = 1;
    let mut cur = code_ptr + 1;
    while count != 0 {
        if code[cur] == '[' {
            count += 1;
        } else if code[cur] == ']' {
            count -= 1;
        }
        cur += 1;
    }
    cur
}

fn matching_open(code: &Vec<char>, code_ptr: usize) -> usize {
    let mut count = 1;
    let mut cur = code_ptr - 1;
    while count != 0 {
        if code[cur] == '[' {
            count -= 1;
        } else if code[cur] == ']' {
            count += 1;
        }
        cur -= 1;
    }
    cur
}
