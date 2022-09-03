// This experiment helped me learn about the ndarray crate, which is a very
// Rust-y way to re-implement numpy.
//
// To experiment with ndarray, I implemented a batched outer product in three
// different ways. The batched outer product takes an input of shape (..., Z)
// and outputs an input of shape (..., Z, Z), where output[..., i, j] is the
// product input[..., i] * input[..., j].

use ndarray::{Array, Axis, Dimension, IntoDimension, Ix3, Ix4, LinalgScalar};
use ndarray_rand::rand_distr::Uniform;
use ndarray_rand::RandomExt;
use std::time::Instant;

fn main() {
    // The RandomExt trait puts this method on ArrayBase which is
    // the type underlying Array.
    let arr = Array::<f32, Ix4>::random((8, 3, 6, 128), Uniform::new(-1., 1.));

    // The mean reduction returns None only for empty arrays.
    println!("input array mean: {:.5}", arr.mean().unwrap());

    // Time a few different techniques for computing the batched
    // outer product, where every axis except the last is considered
    // the "batch" dimension.
    let t1 = Instant::now();
    let naive_out = outer_products_naive(&arr);
    let t2 = Instant::now();
    let fast_out = outer_products_faster(&arr);
    let t3 = Instant::now();
    let bcast_out = outer_products_bcast(&arr);
    let t4 = Instant::now();
    println!("naive method took {:.5} seconds", (t2 - t1).as_secs_f64());
    println!("faster method took {:.5} seconds", (t3 - t2).as_secs_f64());
    println!("bcast method took {:.5} seconds", (t4 - t3).as_secs_f64());

    // Make sure the faster techniques are actually correct.
    println!(
        "faster <-> naive error: {:.5}",
        (naive_out.clone() - fast_out)
            .map(|x| x.abs())
            .mean()
            .unwrap()
    );
    println!(
        "bcast <-> naive error: {:.5}",
        (naive_out - bcast_out).map(|x| x.abs()).mean().unwrap()
    );
}

// Compute a batched outer product by iterating over every element of the
// output, getting the two inputs corresponding to this output, and
// multiplying them.
//
// This is very slow because it requires constructing index tuples for every
// iteration of the inner loop.
fn outer_products_naive<A: LinalgScalar, D: Dimension>(a: &Array<A, D>) -> Array<A, D::Larger> {
    let input_dim = a.raw_dim();
    let d = input_dim.ndim();
    let vec_size = input_dim[d - 1];

    let mut output_dim = D::Larger::zeros(d + 1);
    output_dim.slice_mut()[0..d].clone_from_slice(input_dim.slice());
    output_dim[d] = vec_size;

    let mut res = Array::<A, D::Larger>::zeros(output_dim.clone());
    for (dst_idx_pattern, dst_val) in res.indexed_iter_mut() {
        // The dst_index is something like (X, Y, ..., Z1, Z2), and we want
        // two source indices (X, Y, ..., Z1) and (X, Y, ..., Z2).
        let dst_idx = dst_idx_pattern.into_dimension();
        let mut src_idx = input_dim.clone();
        src_idx.slice_mut().clone_from_slice(&dst_idx.slice()[0..d]);
        let val_1 = a[src_idx.clone()];
        src_idx[d - 1] = dst_idx[d];
        let val_2 = a[src_idx];
        *dst_val = val_1 * val_2;
    }
    res
}

// Compute batched outer products by looping over vectors in the input and
// matrices of the output with zip(), and copying outer products accordingly.
fn outer_products_faster<A: LinalgScalar, D: Dimension>(a: &Array<A, D>) -> Array<A, D::Larger> {
    let d = a.raw_dim().ndim();
    let inner_size = a.shape()[d - 1];
    let outer_size = a.raw_dim().size() / inner_size;

    let mut flat_out = Array::<A, Ix3>::zeros((outer_size, inner_size, inner_size));
    for (src, mut dst) in Iterator::zip(a.rows().into_iter(), flat_out.outer_iter_mut().into_iter())
    {
        let v = src.clone().into_shape((inner_size, 1)).unwrap();
        let v_t = v.clone().reversed_axes();
        dst.assign(&v.dot(&v_t));
    }

    let mut out_shape = <D as Dimension>::Larger::zeros(d + 1);
    out_shape.slice_mut()[0..d].clone_from_slice(a.shape());
    out_shape[d] = inner_size;
    flat_out.into_shape(out_shape).unwrap()
}

// The simplest possible way to compute batched outer products, using broadcast
// semantics. An array of shape (X, Y, ..., Z) can be turned into two views of
// shape (X, Y, ..., 1, Z) and (X, Y, ..., Z, 1). These, when multiplied
// together, create an outer product over the Z axis.
fn outer_products_bcast<A: LinalgScalar, D: Dimension>(a: &Array<A, D>) -> Array<A, D::Larger> {
    let d = a.raw_dim().ndim();
    let arr_1 = a.clone().insert_axis(Axis(d));
    let arr_2 = a.clone().insert_axis(Axis(d - 1));
    return arr_1 * arr_2;
}
