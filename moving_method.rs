// Seeing if into_iter() on a Vec consumes a reference to
// that vec.

fn main() {
    let v = vec![1, -1];
    use_vec(&v);
    use_into_iter(&v);
}

// Works because Vec implements Copy, so v doesn't need to
// be moved to call into_iter() on it.
fn use_vec(v: &Vec<i32>) {
    for x in v {
        println!("{}", x);
    }
    for x in v {
        println!("{}", x)
    }
}

// Does not work, because no Copy trait:
//
//     fn use_into_iter<'a, T: IntoIterator<Item = &'a i32>>(v: T) {
//         for x in v {
//             println!("{}", x);
//         }
//         for x in v {
//             println!("{}", x)
//         }
//     }

// Works, because of Copy trait.
fn use_into_iter<'a, T: IntoIterator<Item = &'a i32> + Copy>(v: T) {
    for x in v {
        println!("{}", x);
    }
    for x in v {
        println!("{}", x)
    }
}
