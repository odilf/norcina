mod event;
pub use event::Event;

mod alg;
pub use alg::Alg;
pub mod mov;
pub use mov::Move;

pub mod math;

// TODO: No need in theory for `T` to be [`Copy`]
pub fn array_map_enumerate<const N: usize, T, R>(
    input: [T; N],
    mut f: impl FnMut(usize, T) -> R,
) -> [R; N]
where
    T: Copy,
{
    std::array::from_fn(|i| f(i, input[i]))
}
