// Experiment to see if all immutable references implement
// Copy and Clone.

struct Foo(i8);

fn main() {
    let a = Foo(3);
    let b = &a;
    let c = foo(b);
    let d = foo1(b);
    println!("{} {} {}", b.0, c.0, d.0);

    // Doesn't work, because Copy isn't implemented for Foo:
    //
    //     foo(a);
    //
}

fn foo<T: Copy>(t: T) -> T {
    return t.clone()
}

fn foo1<T: Clone>(t: T) -> T {
    return t.clone()
}
