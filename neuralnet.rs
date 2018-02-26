// Toy neural network training in Rust.

// An N-dimensional array of floating-point values.
struct Tensor {
    data: Vec<f32>,
    shape: Vec<usize>
}

// A tensor that can be back-propagated through.
trait Res {
    fn value(&self) -> &Tensor;

    fn backward(self, out_grad: &Tensor);
}

struct Sum<A: Res, B: Res> {
    a: A,
    b: B,
    sum: Tensor
}

impl<A: Res, B: Res> Sum<A, B> {
    fn new(a: A, b: B) -> Result<Sum<A, B>, String> {
        if a.value().shape != b.value().shape {
            return Err("input shapes do not match".into());
        }
        let mut sum_data = Vec::<f32>::new();
        let size = a.value().data.len();
        for i in 0..size {
            sum_data.push(a.value().data[i] + b.value().data[i]);
        }
        let tensor = Tensor{data: sum_data, shape: a.value().shape.clone()};
        Ok(Sum{
            a: a,
            b: b,
            sum: tensor
        })
    }
}

impl<A: Res, B: Res> Res for Sum<A, B> {
    fn value(&self) -> &Tensor {
        &self.sum
    }

    fn backward(self, out_grad: &Tensor) {
        self.a.backward(out_grad);
        self.b.backward(out_grad);
    }
}

fn main() {
}
