// Demonstrate a misleading error message.
//
// The error is due to the fact that I'm trying to mutate
// a mutable reference inside an immutable reference.
// The error message, on the other hand, claims to be about
// lifetimes, and says two lifetimes are different even
// though it graphically shows that they are the same:
//
//     error[E0312]: lifetime of reference outlives lifetime of borrowed content...
//       --> incorrect_error.rs:55:32
//        |
//     55 |         let x: &'a mut MyFoo = *self;
//        |                                ^^^^^
//        |
//     note: ...the reference is valid for the lifetime 'a as defined on the body at 53:28...
//       --> incorrect_error.rs:53:29
//        |
//     53 |       fn mutate(&self, i: u8) {
//        |  _____________________________^ starting here...
//     54 | |         // Self is a &&'a mut MyFoo.
//     55 | |         let x: &'a mut MyFoo = *self;
//     56 | |         x.0 = i;
//     57 | |     }
//        | |_____^ ...ending here
//     note: ...but the borrowed content is only valid for the anonymous lifetime #1 defined on the body at 53:28
//       --> incorrect_error.rs:53:29
//        |
//     53 |       fn mutate(&self, i: u8) {
//        |  _____________________________^ starting here...
//     54 | |         // Self is a &&'a mut MyFoo.
//     55 | |         let x: &'a mut MyFoo = *self;
//     56 | |         x.0 = i;
//     57 | |     }
//        | |_____^ ...ending here
//

fn main() {
    bar(&mut MyFoo(17));
}

fn bar<T: MutableFoo>(mut x: T) {
    x.mutate(13)
}

trait MutableFoo {
    fn mutate(&self, i: u8);
}

#[derive(Clone, Copy)]
struct MyFoo(u8);

impl<'a> MutableFoo for &'a mut MyFoo {
    fn mutate(&self, i: u8) {
        // Self is a &&'a mut MyFoo.
        let x: &'a mut MyFoo = *self;
        x.0 = i;
    }
}
