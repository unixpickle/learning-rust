use crate::nat_type::{Inc, Nat, N0};

pub trait Comparison {
    type Neg: Comparison;
    const VALUE: i8;
}

pub struct LessThan;
pub struct GreaterThan;
pub struct Equal;

impl Comparison for LessThan {
    type Neg = GreaterThan;
    const VALUE: i8 = -1;
}

impl Comparison for GreaterThan {
    type Neg = LessThan;
    const VALUE: i8 = 1;
}

impl Comparison for Equal {
    type Neg = Self;
    const VALUE: i8 = 0;
}

pub trait Cmp<T1: Nat> {
    type Result: Comparison;
}

impl Cmp<N0> for N0 {
    type Result = Equal;
}

impl<T: Nat> Cmp<N0> for Inc<T> {
    type Result = GreaterThan;
}

impl<T: Nat> Cmp<Inc<T>> for N0 {
    type Result = LessThan;
}

impl<T1: Nat + Cmp<T2>, T2: Nat> Cmp<Inc<T2>> for Inc<T1> {
    type Result = <T1 as Cmp<T2>>::Result;
}

pub trait Max<T1: Nat> {
    type Max: Nat;
}

impl<T: Nat + Cmp<T1>, T1: Nat> Max<T1> for T
where
    (T, T1, <T as Cmp<T1>>::Result): InnerMax,
{
    type Max = <(T, T1, <T as Cmp<T1>>::Result) as InnerMax>::Result;
}

pub trait Min<T1: Nat> {
    type Min: Nat;
}

impl<T: Nat + Cmp<T1>, T1: Nat> Min<T1> for T
where
    (T, T1, <<T as Cmp<T1>>::Result as Comparison>::Neg): InnerMax,
{
    type Min = <(T, T1, <<T as Cmp<T1>>::Result as Comparison>::Neg) as InnerMax>::Result;
}

pub trait InnerMax {
    type Result: Nat;
}

impl<T1: Nat, T2: Nat> InnerMax for (T1, T2, LessThan) {
    type Result = T2;
}

impl<T1: Nat, T2: Nat> InnerMax for (T1, T2, Equal) {
    type Result = T1;
}

impl<T1: Nat, T2: Nat> InnerMax for (T1, T2, GreaterThan) {
    type Result = T1;
}

// This does not work because of conflicting implementations.
// Workaround from: https://stackoverflow.com/questions/40392524/conflicting-trait-implementations-even-though-associated-types-differ
//
//     impl<T1: Nat + Cmp<T2, Result = GreaterThan>, T2: Nat> Max<T2> for T1 {
//         type Max = T1;
//     }
//
//     impl<T1: Nat + Cmp<T2, Result = Equal>, T2: Nat> Max<T2> for T1 {
//         type Max = T1;
//     }
//
//     impl<T1: Nat + Cmp<T2, Result = LessThan>, T2: Nat> Max<T2> for T1 {
//         type Max = T2;
//     }
