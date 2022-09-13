use std::marker::PhantomData;

// All numbers contains a constant value and know how to add 1 to them via the
// Next type.
pub trait Nat {
    type Next: Nat;
    const VALUE: u64;
}

// Zero is our base case for recursion, so we need a special type for it.
pub struct N0;

impl Nat for N0 {
    type Next = Inc<Self>;
    const VALUE: u64 = 0;
}

// Intuitively, Inc<T> is one greater than T. Thus we can represent any number
// n as Inc<Inc<Inc<...<N0>...>>> where there are n Inc's.
pub struct Inc<T: Nat>(PhantomData<T>);

impl<T: Nat> Nat for Inc<T> {
    type Next = Inc<Self>;
    const VALUE: u64 = T::VALUE + 1;
}

// We create a trait for addition, where for any base type T1, we have the sum
// of T1 and T2 given by <T1 as Add<T2>>::Sum.
pub trait Add<T: Nat> {
    type Sum: Nat;
}

// Base case: adding T to zero gives T.;
impl<T: Nat> Add<T> for N0 {
    type Sum = T;
}

// If we want to add some number T to an Inc<...Inc<N0>...>, then we can
// replace the innermost N0 with T.
//
// To do this, we recursively unwind Inc's until we hit the N0 base case.
impl<Inner: Nat + Add<T>, T: Nat> Add<T> for Inc<Inner> {
    type Sum = Inc<Inner::Sum>;
}

// We create a trait for multiplication, where for any base type T1, we have
// the product of T1 and T2 given by <T1 as Mul<T2>>::Prod.
pub trait Mul<T: Nat> {
    type Prod: Nat;
}

// Base case: multiplying by zero is zero.
impl<T: Nat> Mul<T> for N0 {
    type Prod = N0;
}

// To multiply some Inc chain by T, we add T to itself recursively as we unwind
// the Inc chain, until we arrive at the N0 base case.
//
// The type constraints here are also recursive, since we need to make sure we
// can Add and Mul intermediate results.
impl<Inner: Nat + Mul<T>, T: Nat> Mul<T> for Inc<Inner>
where
    T: Add<<Inner as Mul<T>>::Prod>,
{
    type Prod = <T as Add<<Inner as Mul<T>>::Prod>>::Sum;
}
