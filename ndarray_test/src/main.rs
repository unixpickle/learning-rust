use std::ops::Mul;
use ndarray::{Array, Dimension, IntoDimension, Ix2};
use num_traits::identities::Zero;
fn main() {
    let arr = Array::<f32, Ix2>::from_shape_vec((2, 3), vec![1., 2., 3., 4., 5., 6.]).unwrap();
    println!("{}", arr);
    println!("outer {}", outer_products(arr));
}

// Attempt to make a function that computes the outer product of every vector
// at the last dimension of an N-dimensional array.
// For example, an X x Y x Z tensor would result in an X x Y x Z x Z tensor.
fn outer_products<A: Clone + Zero + Copy + Mul<Output = A>, D: Dimension>(a: Array<A, D>) -> Array<A, D::Larger> {
    let raw_dim = a.raw_dim();
    let mut shape = D::Larger::zeros(raw_dim.ndim() + 1);

    for (i, x) in raw_dim.as_array_view().into_iter().enumerate() {
        shape.as_array_view_mut()[i] = *x;
    }
    let last_dim = shape.as_array_view()[raw_dim.ndim() - 1];
    shape.as_array_view_mut()[raw_dim.ndim()] = last_dim;

    let mut res = Array::<A, D::Larger>::zeros(shape);
    for (idx_1_pattern, val_1) in a.indexed_iter() {
        let idx_1 = idx_1_pattern.into_dimension();
        let mut out_idx = shape.clone();
        for (i, x) in idx_1.as_array_view().into_iter().enumerate() {
            out_idx.as_array_view_mut()[i] = *x;
        }
        for n in 0..last_dim {
            out_idx[raw_dim.ndim()] = n;
            let idx_2 = idx_1.clone();
            idx_2.as_array_view_mut()[idx_2.ndim() - 1] = n;
            let val_2: A = a[idx_2];
            res[out_idx] = (*val_1) * val_2;
        }
    }
    res
}
