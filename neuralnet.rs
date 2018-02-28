// Toy neural network training in Rust.

use std::collections::HashMap;
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

struct Gradient(HashMap<String, Tensor>);

impl Gradient {
    fn new(name: String, value: Tensor) -> Gradient {
        let mut res = Gradient(HashMap::<String, Tensor>::new());
        res.0.insert(name, value);
        res
    }

    fn combine(mut self, other: Gradient) -> Gradient {
        for (k, v) in other.0.into_iter() {
            self.0.insert(k, v);
        }
        self
    }
}

// A tensor that can be back-propagated through.
trait Res {
    fn value(&self) -> &Tensor;

    fn name(&self) -> String;

    fn backward(&mut self, out_grad: &Tensor) -> Gradient;
}

macro_rules! define_op_res {
    ($name:expr, $trait:tt, $fn:tt, $res_name:tt, $bwd:item) => {
        struct $res_name {
            a: Box<Res>,
            b: Box<Res>,
            out: Tensor
        }

        impl Res for $res_name {
            fn value(&self) -> &Tensor {
                &self.out
            }

            fn name(&self) -> String {
                format!("{}<{}, {}>", $name, self.a.name(), self.b.name())
            }

            fn backward(&mut self, out_grad: &Tensor) -> Gradient {
                $bwd;
                // TODO: why doesn't *self.a or self.a.deref_mut() work?
                // TODO: why the static in Box<Res + 'static>?
                bwd(self.a.as_mut(), self.b.as_mut(), out_grad)
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

define_op_res!("Add", Add, add, AddRes,
    fn bwd(a: &mut Res, b: &mut Res, out_grad: &Tensor) -> Gradient {
        a.backward(out_grad).combine(b.backward(out_grad))
    });

define_op_res!("Mul", Mul, mul, MulRes,
    fn bwd(a: &mut Res, b: &mut Res, out_grad: &Tensor) -> Gradient {
        a.backward(&(out_grad * b.value())).combine(b.backward(&(out_grad * a.value())))
    });

define_op_res!("Div", Div, div, DivRes,
    fn bwd(a: &mut Res, b: &mut Res, out_grad: &Tensor) -> Gradient{
        // TODO: figure out why we have to bind b_grad to a
        // variable and then pass it in.
        // I think it's because b.backward() grabs a mutable
        // reference to b before b.value() can run.
        let b_grad = &(-1f32 * &(a.value() * out_grad)) / &(b.value() * b.value());
        a.backward(&(out_grad / b.value())).combine(b.backward(&b_grad))
    });

define_op_res!("Sub", Sub, sub, SubRes,
    fn bwd(a: &mut Res, b: &mut Res, out_grad: &Tensor) -> Gradient {
        a.backward(out_grad).combine(b.backward(&(out_grad * -1f32)))
    });

struct Variable {
    data: Tensor,
    name: String
}

impl Variable {
    fn new(name: String, value: Tensor) -> Variable {
        Variable{
            data: value,
            name: name
        }
    }
}

impl<'a> Res for Variable {
    fn value(&self) -> &Tensor {
        &self.data
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn backward(&mut self, out_grad: &Tensor) -> Gradient {
        Gradient::new(self.name(), out_grad.clone())
    }
}

struct Fork {
    input: Box<Res>,
    output: Box<Res>,
    name1: String,
    name2: String
}

impl Fork {
    fn new<F>(input: Box<Res>, op: F) -> Fork
        where F: FnOnce(Box<Res>, Box<Res>) -> Box<Res>
    {
        let name1 = format!("Fork<{}>[0]", input.name());
        let name2 = format!("Fork<{}>[1]", input.name());
        let in1 = Variable::new(name1.clone(), input.value().clone());
        let in2 = Variable::new(name2.clone(), input.value().clone());
        let output = op(Box::new(in1), Box::new(in2));
        Fork{input: input, output: output, name1: name1, name2: name2}
    }
}

impl Res for Fork {
    fn value(&self) -> &Tensor {
        self.output.value()
    }

    fn name(&self) -> String {
        format!("Forked<{}>", self.output.name())
    }

    fn backward(&mut self, out_grad: &Tensor) -> Gradient {
        let mut grad = self.output.backward(out_grad);
        let summed = match grad.0.remove(&self.name1) {
            Some(grad1) => {
                match grad.0.remove(&self.name2) {
                    Some(grad2) => Some(&grad1 + &grad2),
                    None => Some(grad1)
                }
            },
            None => grad.0.remove(&self.name2)
        };
        match summed {
            Some(downstream) => grad.combine(self.input.backward(&downstream)),
            None => grad
        }
    }
}

fn main() {

}
