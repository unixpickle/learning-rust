use ndarray::{Array, Dimension, IntoDimension, Ix2, Ix3, LinalgScalar};
use num_traits::identities::Zero;
use std::ops::Mul;

fn main() {
    let arr = Array::<f32, Ix2>::from_shape_vec((2, 3), vec![1., 2., 3., 4., 5., 6.]).unwrap();
    println!("input array: {}", arr);
    println!("naive {}", outer_products_naive(arr.clone()));
    println!("faster {}", outer_products_faster(arr.clone()));
}

// Attempt to make a function that computes the outer product of every vector
// at the last dimension of an N-dimensional array.
// For example, an X x Y x Z tensor would result in an X x Y x Z x Z tensor.
fn outer_products_naive<A: Clone + Zero + Copy + Mul<Output = A>, D: Dimension>(
    a: Array<A, D>,
) -> Array<A, D::Larger> {
    let raw_dim = a.raw_dim();
    let mut shape = D::Larger::zeros(raw_dim.ndim() + 1);

    for (i, x) in raw_dim.as_array_view().into_iter().enumerate() {
        shape.as_array_view_mut()[i] = *x;
    }
    let last_dim = shape.as_array_view()[raw_dim.ndim() - 1];
    shape.as_array_view_mut()[raw_dim.ndim()] = last_dim;

    let mut res = Array::<A, D::Larger>::zeros(shape.clone());
    for (idx_1_pattern, val_1) in a.indexed_iter() {
        let idx_1 = idx_1_pattern.into_dimension();
        let mut out_idx = shape.clone();
        for (i, x) in idx_1.as_array_view().into_iter().enumerate() {
            out_idx.as_array_view_mut()[i] = *x;
        }
        for n in 0..last_dim {
            out_idx[raw_dim.ndim()] = n;
            let mut idx_2 = idx_1.clone();
            idx_2.as_array_view_mut()[idx_1.ndim() - 1] = n;
            let val_2: A = a[idx_2];
            res[out_idx.clone()] = (*val_1) * val_2;
        }
    }
    res
}

fn outer_products_faster<A: LinalgScalar, D: Dimension>(a: Array<A, D>) -> Array<A, D::Larger> {
    let mut outer_size: usize = 1;
    let shape_in = a.shape().clone(); // clone since a is consumed later.
    for i in &shape_in[0..shape_in.len() - 1] {
        outer_size *= i;
    }
    let inner_size = shape_in[shape_in.len() - 1];

    let mut flat_out = Array::<A, Ix3>::zeros((outer_size, inner_size, inner_size));
    for (src, mut dst) in Iterator::zip(
        a.genrows().into_iter(),
        flat_out.outer_iter_mut().into_iter(),
    ) {
        let v = src.clone().into_shape((inner_size, 1)).unwrap();
        let v_t = v.clone().reversed_axes();
        dst.assign(&v.dot(&v_t));
    }

    let mut out_shape = <D as Dimension>::Larger::zeros(shape_in.len() + 1);
    for (i, x) in shape_in.into_iter().enumerate() {
        out_shape.as_array_view_mut()[i] = *x;
    }
    out_shape[shape_in.len()] = inner_size;
    flat_out.into_shape(out_shape).unwrap()
}
