// Playing with lifetimes.
//
// Bottom line, I'm seeing some circular reasoning in the
// lifetime system:
// If we have Vec<&'a mut T>, and we have a mutable ref to
// it of the form &'a mut Vec<&mut 'a T>, then it looks
// like we can put the mutable ref INSIDE the vector,
// since the reference has the same lifetime as the vector
// elements.
//
// TODO: see if this works with a non-generic type with an
// explicit lifetime parameter.

// Proving side-effects of the constraints:
//
//  - the lifetime of push's argument is 'a.
//  - the lifetime of push's argument must be at least
//    as long as the lifetime of the referenced Vec.
//  => 'a is at least as long as the lifetime of the
//     referenced Vec.
//
//  - the &mut reference to the Vec lives at least as
//    long as the minimum value for 'a.
//  => the &mut reference lives at least as long as the
//     referenced Vec. This should contradict Rust's
//     entire type system.
//
//  - a reference cannot outlive its referent, so the &mut
//    lives no longer than the referenced Vec.
//  => 'a = the lifetime of the Vec.
//
// In short, using 'a for the first argument constrains
// the first argument to outlive the second argument.
// Using 'a for the second argument constrains the second
// argument to outlive the first argument.
fn add_ref_to_vec<'a>(reference: &'a i32, out: &'a mut Vec<&'a i32>) {
    out.push(reference);

    // Doesn't work, because x forces the result of
    // get_first() to have a shorter lifetime than
    // our function, and 'a must live at least as long
    // as our function.
    //
    //     let x = 3i32;
    //     out.push(get_first(reference, &x));
    //
}

// This function esentially lets us restrict the output
// to the min of two lifetimes, even though we only need
// the first argument to survive.
fn get_first<'a>(input: &'a i32, _: &'a i32) -> &'a i32 {
    input
}

fn main() {
    let mut value = 4i32;
    {
        let mut vec = Vec::<&i32>::new();
        add_ref_to_vec(&value, &mut vec);

        // Don't work because add_ref_to_vec() eats its reference
        // arguments until the vec goes away:
        //
        //     println!("{:?}", vec);
        //     value = 3i32;
        //
    }
    // Works because vec is gone, so there's no
    // references to value anymore.
    value = 3i32;
    println!("{}", value);
}
