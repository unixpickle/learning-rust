// Various ways to dereference a Box.

use std::ops::DerefMut;

struct Foo {
    field: Box<ToString>
}

fn use_foo(f: &mut Foo) -> String {
    // Doesn't work:
    // my_method(*field)

    // Works:
    my_method(&mut *f.field)

    // Works, but needs `use`:
    // my_method(f.field.deref_mut())
}

fn my_method(x: &mut ToString) -> String {
    x.to_string()
}

fn main() {
    println!("{}", use_foo(&mut Foo{field: Box::new(3i32)}));
}
