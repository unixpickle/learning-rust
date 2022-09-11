#![recursion_limit = "10000"]

use std::marker::PhantomData;

type N1 = Inc<N0>;
type N2 = Inc<N1>;
type N3 = Inc<N2>;
type N4 = Inc<N3>;
type N5 = Inc<N4>;
type N6 = Inc<N5>;
type N7 = Inc<N6>;
type N8 = Inc<N7>;
type N9 = Inc<N8>;
type N10 = Inc<N9>;
type N100 = <N10 as Mul<N10>>::Prod;

fn main() {
    println!("2+3 = {}", <N2 as Add<N3>>::Sum::VALUE);
    println!("2+3+4 = {}", <N2 as Add<<N3 as Add<N4>>::Sum>>::Sum::VALUE);
    println!("37 = {}", <<N3 as Mul<N10>>::Prod as Add<N7>>::Sum::VALUE);
    println!(
        "237 = {}",
        <<N2 as Mul<N100>>::Prod as Add<<<N3 as Mul<N10>>::Prod as Add<N7>>::Sum>>::Sum::VALUE
    );
}

trait Nat {
    type Next: Nat;
    const VALUE: u64;
}

struct N0;

impl Nat for N0 {
    type Next = Inc<Self>;
    const VALUE: u64 = 0;
}

struct Inc<T: Nat>(PhantomData<T>);

impl<T: Nat> Nat for Inc<T> {
    type Next = Inc<Self>;
    const VALUE: u64 = T::VALUE + 1;
}

trait Add<T: Nat> {
    type Sum: Nat;
}

impl<T: Nat> Add<T> for N0 {
    type Sum = T;
}

impl<Inner: Nat + Add<T>, T: Nat> Add<T> for Inc<Inner> {
    type Sum = Inc<Inner::Sum>;
}

trait Mul<T: Nat> {
    type Prod: Nat;
}

impl<T: Nat> Mul<T> for N0 {
    type Prod = N0;
}

impl<Inner: Nat + Mul<T>, T: Nat> Mul<T> for Inc<Inner>
where
    T: Add<<Inner as Mul<T>>::Prod>,
{
    type Prod = <T as Add<<Inner as Mul<T>>::Prod>>::Sum;
}
