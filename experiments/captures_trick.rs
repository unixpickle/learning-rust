// Investigate using the captures trick to capture two lifetimes in Rust.
//
// Example taken directly from this blog post:
// https://blog.rust-lang.org/2024/10/17/Rust-1.82.0.html

struct Ctx<'cx>(&'cx u8);

trait Captures<A, B> {}

impl<A, B, C> Captures<A, B> for C {}

// This version doesn't compile:
//
// fn f<'cx, 'a>(cx: Ctx<'cx>, x: &'a u8) -> impl Iterator<Item = &'a u8> + 'cx {
//     core::iter::once_with(move || {
//         eprintln!("LOG: {}", cx.0);
//         x
//     })
// }

fn f<'cx, 'a>(cx: Ctx<'cx>, x: &'a u8) -> impl Iterator<Item = &'a u8> + Captures<&'cx (), &'a ()> {
    core::iter::once_with(move || {
        eprintln!("LOG: {}", cx.0);
        x
    })
}

fn main() {
    let ctx_val: u8 = 3;
    let ctx = Ctx(&ctx_val);
    let x: u8 = 4;
    for x in f(ctx, &x) {
        println!("got value {}", x);
    }
}
