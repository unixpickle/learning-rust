# learning-rust

A bunch of random stuff I write as I try to learn Rust.

# Table of Contents

 * [bf.rs](bf.rs) - a [Brainfuck](https://en.wikipedia.org/wiki/Brainfuck) interpreter.
 * [solve_2x2.rs](solve_2x2.rs) - a 2x2 Pocket Cube solver.
 * [autodiff.rs](autodiff.rs) - a small automatic-differentiation program.
 * [game_of_life.rs](game_of_life.rs) - Conway's [game of life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life) implementation.
 * [experiments](experiments) - misc. experiments to improve my mental model of Rust.
 * [http_proxy](http_proxy/src/main.rs) - a program that forwards HTTP requests to another host. Logs every request and the total number of bytes read and written across all requests.
 * [spellingbee_answers](spellingbee_answers/src) - a web scraper to get the answers out of the NYT Spelling Bee game. This uses multiple files with modules, and a custom error type. It also does some time arithmetic to fetch new answers whenever the game swaps.
 * [type_math](type_math/src) - compile-type arithmetic using the Rust type system. In particular, it implements Peano numbers, which in this case represent a number by the number of type recursions.
 * [futures_test](futures_test/src/main.rs) - implementing futures and/or executors from scratch, and comparing to third-party versions.
 * [ndarray_test](ndarray_test/src/main.rs) - trying out the ndarray crate, and benchmarking various ways of computing an operation with it.
