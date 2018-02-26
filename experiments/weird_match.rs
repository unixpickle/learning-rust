// Demonstrating a weird syntax for match default cases.

fn main() {
    let x = 3;
    match x {
        1 => println!("1"),
        2 => println!("2"),
        a => println!("{}!!!!", a),
    }
}
