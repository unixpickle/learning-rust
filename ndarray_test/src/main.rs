use ndarray::{Array, Dimension, Ix2};
use num_traits::identities::Zero;
fn main() {
    let arr = Array::<f32, Ix2>::from_shape_vec((2, 3), vec![1., 2., 3., 4., 5., 6.]).unwrap();
    println!("{}", arr);
    println!("outer {}", outer_products(arr));
}

// Attempt to make a function that computes the outer product of every vector
// at the last dimension of an N-dimensional array.
// For example, an X x Y x Z tensor would result in an X x Y x Z x Z tensor.
fn outer_products<A: Clone + Zero, D: Dimension>(a: Array<A, D>) -> Array<A, D::Larger> {
    let raw_dim = a.raw_dim();
    let mut shape = D::Larger::zeros(raw_dim.ndim() + 1);

    for (i, x) in raw_dim.as_array_view().into_iter().enumerate() {
        shape.as_array_view_mut()[i] = *x;
    }
    shape.as_array_view_mut()[raw_dim.ndim()] = shape.as_array_view()[raw_dim.ndim() - 1];

    Array::<A, D::Larger>::zeros(shape)
}
