// Toy neural network training in Rust.

use std::ops::{Add, Mul, Div, Sub};

// An N-dimensional array of floating-point values.
#[derive(Clone)]
struct Tensor {
    data: Vec<f32>,
    shape: Vec<usize>
}

impl Tensor {
    fn new(shape: Vec<usize>) -> Tensor {
        let mut size: usize = 1;
        for x in shape.clone() {
            size *= x;
        }
        let mut res = Tensor{data: Vec::<f32>::new(), shape: shape};
        for _ in 0..size {
            res.data.push(0f32);
        }
        res
    }
}

macro_rules! define_tensor_op {
    ($trait:tt, $fn:tt) => {
        impl<'a, 'b> $trait<&'b Tensor> for &'a Tensor {
            type Output = Tensor;

            fn $fn(self, rhs: &'b Tensor) -> Tensor {
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

        impl<'a> $trait<f32> for &'a Tensor {
            type Output = Tensor;

            fn $fn(self, rhs: f32) -> Tensor {
                let mut result = Tensor{data: Vec::<f32>::new(), shape: self.shape.clone()};
                for i in 0..self.data.len() {
                    result.data.push($trait::$fn(self.data[i], rhs));
                }
                result
            }
        }

        impl<'a> $trait<&'a Tensor> for f32 {
            type Output = Tensor;

            fn $fn(self, rhs: &'a Tensor) -> Tensor {
                let mut result = Tensor{data: Vec::<f32>::new(), shape: rhs.shape.clone()};
                for i in 0..rhs.data.len() {
                    result.data.push($trait::$fn(self, rhs.data[i]));
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

macro_rules! define_op_res {
    ($trait:tt, $fn:tt, $res_name:tt, $bwd:item) => {
        struct $res_name {
            a: Box<Res>,
            b: Box<Res>,
            out: Tensor
        }

        impl Res for $res_name {
            fn value(&self) -> &Tensor {
                &self.out
            }

            fn backward(&mut self, out_grad: &Tensor) {
                $bwd;
                // TODO: why doesn't *self.a or self.a.deref_mut() work?
                // TODO: why he static in Box<Res + 'static>?
                bwd(self.a.as_mut(), self.b.as_mut(), out_grad);
            }
        }

        impl $trait for Box<Res> {
            type Output = Box<Res>;

            fn $fn(self, rhs: Box<Res>) -> Box<Res> {
                let out = $trait::$fn(self.value(), rhs.value());
                Box::new($res_name{a: self, b: rhs, out: out})
            }
        }
    }
}

define_op_res!(Add, add, AddRes, fn bwd(a: &mut Res, b: &mut Res, out_grad: &Tensor) {
    a.backward(out_grad);
    b.backward(out_grad);
});

define_op_res!(Mul, mul, MulRes, fn bwd(a: &mut Res, b: &mut Res, out_grad: &Tensor) {
    a.backward(&(out_grad * b.value()));
    b.backward(&(out_grad * a.value()));
});

define_op_res!(Div, div, DivRes, fn bwd(a: &mut Res, b: &mut Res, out_grad: &Tensor) {
    a.backward(&(out_grad / b.value()));
    // TODO: figure out why we have to find b_grad to a
    // variable and then pass it in.
    // I think it's because b.backward() grabs a mutable
    // reference to b before b.value() can run.
    let b_grad = &(-1f32 * &(a.value() * out_grad)) / &(b.value() * b.value());
    b.backward(&b_grad);
});

define_op_res!(Sub, sub, SubRes, fn bwd(a: &mut Res, b: &mut Res, out_grad: &Tensor) {
    a.backward(out_grad);
    b.backward(&(out_grad * -1f32));
});

struct Variable {
    data: Tensor,
    grad: Tensor
}

impl Variable {
    fn new(value: Tensor) -> Variable {
        let grad = Tensor::new(value.shape.clone());
        Variable{
            data: value,
            grad: grad
        }
    }
}

impl<'a> Res for &'a mut Variable {
    fn value(&self) -> &Tensor {
        &self.data
    }

    fn backward(&mut self, out_grad: &Tensor) {
        self.grad = &self.grad + out_grad;
    }
}

fn main() {

}
