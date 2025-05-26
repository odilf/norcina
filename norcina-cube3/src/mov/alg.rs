pub mod algs {
    macro_rules! declare_alg {
        ($name:tt = $($alg:tt)*) => {
            declare_alg!(@inner
                (name=$name)
                (length=0)
                (acc=[])
                (rest=[$($alg),*])
            );
        };

        (@inner (name=$name:ident) (length=$length:expr) (acc=[$($alg:tt),*]) (rest=[ ])) => {
            use $crate::mov::moves::*;
            pub const $name: [Move; $length] = [$($alg),*];
        };

        (@inner
            (name=$name:tt)
            (length=$length:expr)
            (acc=[$($acc:tt)*])
            (rest=[$alg_head:tt $(, $($alg_tail:tt)+)? ])
        ) => {
            declare_alg!(@inner
                (name=$name)
                (length=$length + 1)
                (acc=[$($acc)* $alg_head])
                (rest=[ $($($alg_tail),+)? ])
            );
        };

    }

    use norcina_cube_n::alg;

    use crate::Move;

    // declare_alg!(SIMPLE = R);
    // declare_alg!(@inner (name=SEXY) (length=0) (acc=[]) (rest=[R, U]));
    // declare_alg!(@inner (name=SEXY) (length=0 + 1) (acc=[R]) (rest=[U]));
    // declare_alg!(@inner (name=SEXY) (length=0 + 1 + 1) (acc=[R, U]) (rest=[ ]));

    // declare_alg!(SEXY = R U);
    // declare_alg!(SEXY = R U RP UP);
    // pub const SEXY: [Move; 4] = alg![R U RP UP];
    pub const SLEDGEHAMMER: [Move; 4] = alg!(RP F R FP);

    pub mod oll {
        use super::*;
        pub const T: [Move; 14] = alg!(R U RP UP RP F R2 UP RP UP R U RP FP);
        pub const J: [Move; 13] = alg!(R U RP F R U RP UP RP FP R2 UP RP);
        pub const U: [Move; 11] = U_A;
        pub const U_A: [Move; 11] = alg!(R2 UP RP UP R U R U R UP R);
        pub const U_B: [Move; 11] = alg!(RP U RP UP RP UP RP U R U R2);
    }

    pub const CHECKER: [Move; 6] = alg!(R2 L2 U2 D2 F2 B2);

    // TODO: Concat or extend algs
    // pub const J_AUF: [Move; 14] = [J, alg!(UP)].concat();
}
