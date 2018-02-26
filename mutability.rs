// Various experiments with references and mutability.
//
// Lessons to be learned:
// - A value won't be mutated as long as its bound immutably.
// - When a value is moved, its old binding  ^^^^^ goes away.

fn main() {
    // Not allowed: mutating an immutable variable:
    //
    //     let v = Vec::<i8>::new();
    //     v.push(3);
    //

    // Allowed: moving from an immutable to a mutable
    // variable in order to mutate.
    let a = Vec::<i8>::new();
    let mut b = a;
    b.push(13);
    println!("b: {:?}", b);

    // Even though v is declared as immutable, it can be
    // mutated by a different function if we relinquish
    // ownership of it.
    let v = Vec::<i8>::new();
    foo(v);

    // Doesn't work, because we don't implement MutableFoo for &mut MyFoo:
    //
    //     let mut x = MyFoo(17);
    //     bar(&mut x);
    //

    // Works, because MyFoo implements MutableFoo.
    bar(&mut MyFoo(17));
}

fn foo(v: Vec<i8>) {
    // If an argument is immutable, you can mutate it by
    // moving it first.
    let mut u = v;
    u.push(3);
    println!("foo: {:?}", u);
}

fn bar<T: MutableFoo>(mut x: T) {
    x.mutate(13)
}

trait MutableFoo {
    fn mutate(&mut self, i: u8);
}

struct MyFoo(u8);

impl<'a> MutableFoo for &'a mut MyFoo {
    fn mutate(&mut self, i: u8) {
        // Self is a &mut &'a mut MyFoo.
        // Note: if we had &self as the argument, then
        // we wouldn't be able to mutate *self even though
        // it's supposedly mut.
        self.0 = i;
    }
}
