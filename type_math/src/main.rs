#![recursion_limit = "10000"]

mod comparison;
mod constants;
mod macros;
mod nat_type;
use crate::comparison::{Cmp, Comparison, Max, Min};
use crate::constants::*;
use crate::macros::{add, digits, max, min, mul};
use crate::nat_type::{Add, Mul, Nat};

fn main() {
    println!("--- multiplication and addition ---");
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

    // We can also compare numbers and get min/max.
    println!("--- comparison ---");
    println!(
        "sign(7-3) = {}",
        <<N7 as Cmp<N3>>::Result as Comparison>::VALUE
    );
    println!(
        "sign(7-4) = {}",
        <<N7 as Cmp<N4>>::Result as Comparison>::VALUE
    );
    println!(
        "sign(4-7) = {}",
        <<N4 as Cmp<N7>>::Result as Comparison>::VALUE
    );
    println!(
        "sign(4-4) = {}",
        <<N4 as Cmp<N4>>::Result as Comparison>::VALUE
    );
    println!("min(7,3) = {}", <min!(N7, N3)>::VALUE);
    println!("max(7,3) = {}", <max!(N7, N3)>::VALUE);
    println!("max(7,7) = {}", <max!(N7, N7)>::VALUE);
    println!("min(7,7) = {}", <min!(N7, N7)>::VALUE);
}
