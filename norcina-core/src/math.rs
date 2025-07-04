//! Mostly generic combinatoric functions.

/// The factorial function: n * (n - 1) * (n - 2) * ... * 1
pub const fn fac(n: usize) -> usize {
    if n == 0 { 1 } else { n * fac(n - 1) }
}

/// `n` choose `m`. That is, n! / (m! * (n - m)!)
pub const fn choose(n: usize, m: usize) -> usize {
    fac(n) / fac(m) / fac(n - m)

    // let mut output = 1;

    // // n * (n - 1) * (n - 2) * ... * (n - m + 1)
    // let mut i = n;
    // while i > n - m {
    //     output *= i;
    //     i -= 1;
    // }

    // // m * (m - 1) * (m - 2) * ... * 1
    // let mut i = m;
    // while i > 1 {
    //     output *= i;
    //     i -= 1;
    // }

    // output
}
