use std::rc::Rc;
use std::sync::Arc;

pub trait Bacchiatore {
    fn elo(&self) -> i32;

    fn total_duels(&self) -> u32;

    fn total_days(&self) -> u32;

    fn elo_delta_callback(&self, elo_delta: i32);
}

macro_rules! impl_bacchiatore {
    ($($id:ident),*) => {
        $(
            impl<T: Bacchiatore> Bacchiatore for $id<T> {
                impl_bacchiatore!(@impl);
            }
        )*
    };
    (@impl) => {
        fn elo(&self) -> i32 {
            (**self).elo()
        }

        fn total_duels(&self) -> u32 {
            (**self).total_duels()
        }

        fn total_days(&self) -> u32 {
            (**self).total_days()
        }

        fn elo_delta_callback(&self, elo_delta: i32) {
            (**self).elo_delta_callback(elo_delta);
        }
    };
}

impl<T: Bacchiatore> Bacchiatore for &T {
    impl_bacchiatore!(@impl);
}

impl<T: Bacchiatore> Bacchiatore for &mut T {
    impl_bacchiatore!(@impl);
}

impl_bacchiatore!(Box, Rc, Arc);

pub trait Duel {
    fn equal_points(&self) -> i32;

    fn opposite_points(&self) -> i32;

    fn equal_elo_delta_callback(&self, elo_delta: i32);

    fn opposite_elo_delta_callback(&self, elo_delta: i32);
}

macro_rules! impl_duel {
    ($($id:ident),*) => {
        $(
            impl<T: Duel> Duel for $id<T> {
                impl_duel!(@impl);
            }
        )*
    };
    (@impl) => {
        fn equal_points(&self) -> i32 {
            (**self).equal_points()
        }

        fn opposite_points(&self) -> i32 {
            (**self).opposite_points()
        }

        fn equal_elo_delta_callback(&self, elo_delta: i32) {
            (**self).equal_elo_delta_callback(elo_delta);
        }

        fn opposite_elo_delta_callback(&self, elo_delta: i32) {
            (**self).opposite_elo_delta_callback(elo_delta);
        }
    };
}

impl<T: Duel> Duel for &T {
    impl_duel!(@impl);
}

impl<T: Duel> Duel for &mut T {
    impl_duel!(@impl);
}

impl_duel!(Box, Rc, Arc);
