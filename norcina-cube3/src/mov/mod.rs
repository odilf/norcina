pub mod alg;

pub use alg::algs;

#[cfg(all(test, feature = "quickcheck"))]
mod tests {
    pub use super::*;
    use crate::cube::Cube;
    use norcina_core::mov::InvertibleMove;
    use norcina_cube_n::{
        alg,
        math::Face,
        mov::{Amount, Move},
    };
    use quickcheck::quickcheck;

    quickcheck! {
        fn fn_move_constructor_and_accessors_maintain_values(face: Face, amount: Amount) -> bool {
            let mov = Move::new(face, amount);
            mov.face() == face && mov.amount() == amount
        }

        fn fn_double_double_identity(cube: Cube, face: Face) -> bool {
            let mov = Move::new(face, Amount::Double);
            cube.mov([mov, mov]) == cube
        }

        fn fn_rrp_identity(cube: Cube) -> bool {
            cube.mov(alg!(R RP)) == cube
        }

        fn fn_move_reverse_identity(cube: Cube, face: Face) -> bool {
            let m1 = Move::new(face, Amount::Single);
            let m2 = Move::new(face, Amount::Reverse);
            cube.mov([m1, m2]) == cube
        }

        fn fn_double_t_identity(cube: Cube) -> bool {
            cube.mov(alg!(R RP)) == cube
        }

        fn fn_single_double_equals_reverse(cube: Cube) -> bool {
            cube.mov(alg!(R R2)) == cube.mov(alg!(RP))
        }

        fn move_and_reverse_identity(cube: Cube, mov: Move) -> bool {
            cube.mov_single(mov).mov_single(mov.inverse()) == cube
        }
    }

    #[test]
    fn instance_all_basic_moves() {
        for mov in Move::iter() {
            println!("Move is {mov}");
            insta::assert_debug_snapshot!(Cube::SOLVED.mov_single(mov))
        }
    }

    #[test]
    fn ua_ub_cancel() {
        assert!(
            Cube::SOLVED
                .mov(algs::pll::U_A)
                .mov(algs::pll::U_B)
                .is_solved()
        )
    }
}
