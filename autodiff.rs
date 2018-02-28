// Reverse-mode automatic differentiation.
//
// Components of the system:
//  - Tensor: a shaped vector of floats.
//  - Gradient: a mapping of variable names to grads.
//  - Res: an abstract differentiable value.
//  - Variable: a Res that returns its upstream gradient.
//  - Fork: a Res that allows you to use a Box<Res> more
//    than once, since operations on Box<Res> normally
//    consume the operands. The Fork also accumulates
//    upstream gradients to avoid double-backprop.
//  - Constant: a hacky Res with a constant value.
//
// In general, all nodes in the graph are supposed to have
// a unique identifier (otherwise, Fork wouldn't be able
// to uniquely identify its quasi-variable).
// In other languages, this wouldn't be necessary, since a
// reference to a Variable (e.g. a pointer) could uniquely
// identify it. It might be possible to do this by using
// generics instead of trait objects, although that would
// result in a TON of types being created.

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
    names: Vec<String>
}

impl Fork {
    fn new<F>(input: Box<Res>, n: usize, op: F) -> Fork
        where F: FnOnce(Vec<Box<Res>>) -> Box<Res>
    {
        let mut names = Vec::<String>::new();
        let mut vars = Vec::<Box<Res>>::new();
        for i in 0..n {
            let name = format!("Fork<{}>[{}]", input.name(), i);
            vars.push(Box::new(Variable::new(name.clone(), input.value().clone())));
            names.push(name);
        }
        let output = op(vars);
        Fork{input: input, output: output, names: names}
    }

    fn fork<F>(input: Box<Res>, n: usize, op: F) -> Box<Res>
        where F: FnOnce(Vec<Box<Res>>) -> Box<Res>
    {
        Box::new(Fork::new(input, n, op))
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
        let mut summed: Option<Tensor> = None;
        for name in &self.names {
            match grad.0.remove(name) {
                Some(tensor) => {
                    summed = Some(match summed {
                        Some(tensor1) => &tensor + &tensor1,
                        None => tensor
                    });
                },
                None => ()
            }
        }
        match summed {
            Some(downstream) => grad.combine(self.input.backward(&downstream)),
            None => grad
        }
    }
}

struct Constant(Tensor, String);

impl Constant {
    fn new(shape: Vec<usize>, value: f32) -> Constant {
        let mut res = Constant(Tensor::new(shape), format!("{}", value));
        for i in 0..res.0.data.len() {
            res.0.data[i] = value;
        }
        res
    }
}

impl Res for Constant {
    fn value(&self) -> &Tensor {
        &self.0
    }

    fn name(&self) -> String {
        self.1.clone()
    }

    fn backward(&mut self, _: &Tensor) -> Gradient {
        Gradient(HashMap::<String, Tensor>::new())
    }
}

fn main() {
    // Approximate sin(x) for x=0, x=0.2, x=0.4.
    let x = Variable::new("x".to_string(),
        Tensor{shape: vec![3], data: vec![0f32, 0.2f32, 0.4f32]});
    let shape = x.value().shape.clone();
    let mut sin = Fork::fork(Box::new(x), 5, |mut xs| {
        let x2 = xs.pop().expect("no field") * xs.pop().expect("no field");
        Fork::fork(x2, 3, |mut x2s| {
            let x1 = xs.pop().expect("no field");
            let x3 = xs.pop().expect("no field") * x2s.pop().expect("no field");
            let x5 = xs.pop().expect("no field") * x2s.pop().expect("no field") *
                x2s.pop().expect("no field");
            x1 - (x3 / Box::new(Constant::new(shape.clone(), 6f32))) +
                (x5 / Box::new(Constant::new(shape, 120f32)))
        })
    });
    println!("sin(0, 0.2, 0.4): {:?}", sin.value().data);
    let out_grad = Tensor{shape: vec![3], data: vec![1f32, 1f32, 1f32]};
    println!("cos(0, 0.2, 0.4): {:?}", sin.backward(&out_grad).0["x"].data)
}
