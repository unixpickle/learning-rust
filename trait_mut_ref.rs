// If you implement a trait for type T, but all you have
// is a mutable reference &mut T, you can still call trait
// methods, but you can't pass the &mut T to methods that
// expect the trait. That is, not without a hack like my
// TraitDereferencer.

struct Foo(i8);

fn main() {
    let mut x = Foo(13);
    let y = &mut x;

    // Works:
    y.printme();

    // Doesn't work:
    //bar(y);

    // Works:
    bar(TraitDereferencer(y))
}

fn bar<T: MePrinter>(x: T) {
    x.printme();
}

trait MePrinter {
    fn printme(&self);
}

impl MePrinter for Foo {
    fn printme(&self) {
        println!("MePrinter printme: {}", self.0);
    }
}

struct TraitDereferencer<'a, T: 'a + MePrinter>(&'a mut T);

impl<'a, T: 'a + MePrinter> MePrinter for TraitDereferencer<'a, T> {
    fn printme(&self) {
        self.0.printme();
    }
}
