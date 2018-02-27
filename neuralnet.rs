// Toy neural network training in Rust.

use std::ops::Add;

// An N-dimensional array of floating-point values.
#[derive(Clone)]
struct Tensor {
    data: Vec<f32>,
    shape: Vec<usize>
}

macro_rules! tensor_op {
    ($t1:expr, $op:tt, $t2:expr) => {
        {
            let t1 = $t1;
            let t2 = $t2;
            let mut result = Tensor{data: Vec::<f32>::new(), shape: t1.shape.clone()};
            if t1.shape != t2.shape || t1.data.len() != t2.data.len() {
                panic!("shape mismatch");
            }
            for i in 0..t1.data.len() {
                result.data.push(t1.data[i] $op t2.data[i]);
            }
            result
        }
    }
}

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
        let tensor = tensor_op!(self.value(), +, b.value());
        Box::new(Sum{
            a: self,
            b: b,
            sum: tensor
        })
    }
}

fn main() {
}
