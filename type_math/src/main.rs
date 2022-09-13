#![recursion_limit = "10000"]

mod constants;
mod macros;
mod nat_type;
use crate::constants::*;
use crate::macros::{add, digits, mul};
use crate::nat_type::{Add, Mul, Nat};

fn main() {
    // Adding values happens at compile time.
    println!("2+3+4 = {}", <add!(add!(N2, N3), N4)>::VALUE);

    // We can nest computations as expected.
    println!("3*10 + 7 = {}", <add!(mul!(N3, N10), N7)>::VALUE);

    // We can construct multi-digit numbers using a macro, which uses
    // multiplication and addition internally.
    println!("732 = {}", <digits!([N7, N3, N2])>::VALUE);

    // I couldn't solve this in my head; could you?
    // The rust compiler can, in about 1 second.
    println!(
        "13*82 = {}",
        <mul!(digits!([N1, N3]), digits!([N8, N2]))>::VALUE
    );
}
