// Playing with lifetimes.
//
// An observation: a reference to an object can have the
// exact same lifetime constraint as that object.

struct UncopyableInt(i32);

struct RefOwner<'a>(&'a UncopyableInt);

// No reason to specify lifetime; only one ref arg.
fn return_ref(x: &UncopyableInt) -> RefOwner {
    RefOwner(x)
}

// Doesn't compile without lifetime parameters.
fn return_ref_explicit<'a, 'b>(_: &'a UncopyableInt, y: &'b UncopyableInt) -> RefOwner<'b> {
    RefOwner(y)
}

// Doesn't compile without lifetime parameters, since
// there's no argument to borrow from.
// We could also return `RefOwner<'static>`.
fn make_ref<'a>(i: i32) -> RefOwner<'a> {
    // Does not work, since not static:
    // RefOwner(&UncopyableInt(i))

    // Works, because the UncopyableInt is 'static:
    // RefOwner(&UncopyableInt(1337i32))

    return_ref_explicit(&UncopyableInt(i), &UncopyableInt(1337i32))
}

// This constrains both arguments to have the same lifetime.
//
// Struct<'a> lifetimes restrict 'a to be the exact lifetime
// of the structure. This is different than for references,
// where `&'a var` restricts 'a to be no longer than the
// lifetime of var, but 'a may be shorter than the lifetime
// of var.
//
// Logically, RefOwner<'a> cannot last longer than 'a,
// because then it's field could become invalid.
// It can't last less long than 'a, because it's field has
// a lifetime of 'a as well, meaning that it's field is not
// guaranteed to live past the struct.
fn transfer_contents<'a>(a: &mut RefOwner<'a>, b: &mut RefOwner<'a>) {
    b.0 = a.0;
}

fn use_transfer<'a>(i: i32) {
    // Doesn't work because refA must live as long as
    // refB, and vice versa.
    //
    //     let a = UncopyableInt(i);
    //     let mut refA = RefOwner(&a);
    //     {
    //         let b = UncopyableInt(5i32);
    //         let mut refB = RefOwner(&b);
    //         transfer_contents(&mut refA, &mut refB);
    //         println!("{}", (b.0));
    //     }

    // Works, because both ref_a and ref_b have static
    // content lifetimes:
    let mut ref_a = RefOwner(&UncopyableInt(2i32));
    {
        let mut ref_b = RefOwner(&UncopyableInt(5i32));
        transfer_contents(&mut ref_a, &mut ref_b);
        println!("{}", ((ref_b.0).0));
    }
}

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
    let x: RefOwner;
    {
        // Doesn't work, because y is on the stack:
        // let y = UncopyableInt(5i32);
        // x = return_ref(&y);

        // Works, because the UncopyableInt is static:
        x = return_ref(&UncopyableInt(1337i32));
    }
    println!("from inside scope: {:?}", (x.0).0);
    println!("from inside function: {:?}", (make_ref((x.0).0).0).0);

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
