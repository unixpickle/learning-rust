// Toy neural network training in Rust.

use std::ops::{Add, Mul, Div, Sub};

// An N-dimensional array of floating-point values.
#[derive(Clone)]
struct Tensor {
    data: Vec<f32>,
    shape: Vec<usize>
}

macro_rules! define_tensor_op {
    ($trait:tt, $fn:tt) => {
        impl<'a> $trait<&'a Tensor> for &'a Tensor {
            type Output = Tensor;

            fn $fn(self, rhs: &Tensor) -> Tensor {
                let mut result = Tensor{data: Vec::<f32>::new(), shape: self.shape.clone()};
                if self.shape != rhs.shape || self.data.len() != rhs.data.len() {
                    panic!("shape mismatch");
                }
                for i in 0..self.data.len() {
                    result.data.push($trait::$fn(self.data[i], rhs.data[i]));
                }
                result
            }
        }
    }
}

define_tensor_op!(Add, add);
define_tensor_op!(Mul, mul);
define_tensor_op!(Div, div);
define_tensor_op!(Sub, sub);

// A tensor that can be back-propagated through.
trait Res {
    fn value(&self) -> &Tensor;

    fn backward(&mut self, out_grad: &Tensor);
}

struct Sum {
    a: Box<Res>,
    b: Box<Res>,
    sum: Tensor
}

impl Res for Sum {
    fn value(&self) -> &Tensor {
        &self.sum
    }

    fn backward(&mut self, out_grad: &Tensor) {
        self.a.backward(out_grad);
        self.b.backward(out_grad);
    }
}

impl Add for Box<Res> {
    type Output = Box<Res>;

    fn add(self, b: Box<Res>) -> Box<Res> {
        let tensor = self.value() + b.value();
        Box::new(Sum{
            a: self,
            b: b,
            sum: tensor
        })
    }
}

fn main() {
}
