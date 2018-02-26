// Demonstration that you can't pass &T into functions that
// expect a trait implemented by T.
//
// If you implement a trait for type T, but all you have
// is a reference &T, you can still call trait methods, but
// you can't pass the &T to methods that expect the trait.
// That is, not without a hack like my TraitDereferencer!

use std::fmt::Debug;
use std::ops::Index;

fn main() {
    let mut x = Vec::<u8>::new();
    x.push(13);
    try_to_print_first(&x);
}

fn try_to_print_first(x: &Vec<u8>) {
    // Doesn't work:
    //print_first(x);

    // Works:
    print_first(TraitDereferencer(x));
}

fn print_first<T: Index<usize>>(x: T) where T::Output: Debug + Sized {
    println!("{:?}", x[0])
}

struct TraitDereferencer<'a, T: 'a + Index<usize>>(&'a T) where T::Output: Debug + Sized;

impl<'a, T: 'a + Index<usize>> Index<usize> for TraitDereferencer<'a, T>
    where T::Output: Debug + Sized
{
    type Output = T::Output;
    fn index(&self, index: usize) -> &T::Output {
        self.0.index(index)
    }
}
