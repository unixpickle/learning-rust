// Avoid a ton of nested <> if possible, by implementing
// simple macros for arithmetic.

macro_rules! add {
    ($x:ty, $y:ty) => {
        <$x as Add<$y>>::Sum
    };
}

macro_rules! mul {
    ($x:ty, $y:ty) => {
        <$x as Mul<$y>>::Prod
    };
}

// Construct multi-digit numbers using the standard trick of repeatedly
// multiplying by 10 and adding the next digit.
//
// Ideally, macros would allow us to recurse on the last argument rather than
// the first. However, they do not support backtracking, so this doesn't work
// naively. As a workaround, we first reverse the arguments and then compute
// the result. To do this, the macro needs to have two separate sections, one
// forward and one reversed. We put the forward part in brackets, as done by:
// https://stackoverflow.com/questions/42171483/how-to-recursively-take-the-last-argument-of-a-macro
macro_rules! digits {
    // Both rules for when the reversing is complete.
    ([] $x:ty) => { $x };
    ([] $x:ty, $($y:ty),+) => {
        add!(mul!(digits!([] $($y),*), N10), $x)
    };

    // Reverse arguments in the brackets.
    ([$first:ty $(,$rest:ty)*] $($reversed:ty),*) => {
        digits!([$($rest),*] $first $(,$reversed)*)
    };
}

pub(crate) use add;
pub(crate) use digits;
pub(crate) use mul;
