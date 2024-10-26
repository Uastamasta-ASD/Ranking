/// Trait representing a bacchiatore.
pub trait Bacchiatore {
    /// Returns the current elo of the bacchiatore.
    fn elo(&self) -> i32;

    /// Returns the number of *already-ranked* duels played by this bacchiatore.
    fn total_duels(&self) -> u32;

    /// Returns the number of days in which this bacchiatore played an *already-ranked* duels.
    fn total_days(&self) -> u32;

    /// Called at the end of the elo calculation with the elo increase/decrease for this bacchiatore.
    fn elo_gain_callback(&mut self, elo_gain: i32);
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

        fn elo_gain_callback(&mut self, elo_gain: i32) {
            (**self).elo_gain_callback(elo_gain);
        }
    };
}

impl<T: Bacchiatore> Bacchiatore for &mut T {
    impl_bacchiatore!(@impl);
}

impl_bacchiatore!(Box);

/// Trait representing a duel.
pub trait Duel {
    /// Returns the points done by equal.
    fn equal_points(&self) -> i32;

    /// Returns the points done by opposite.
    fn opposite_points(&self) -> i32;

    /// Called at the end of the elo calculation with the elo increase/decrease for equal.
    fn equal_elo_gain_callback(&mut self, elo_gain: i32);

    /// Called at the end of the elo calculation with the elo increase/decrease for opposite.
    fn opposite_elo_gain_callback(&mut self, elo_gain: i32);
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

        fn equal_elo_gain_callback(&mut self, elo_gain: i32) {
            (**self).equal_elo_gain_callback(elo_gain);
        }

        fn opposite_elo_gain_callback(&mut self, elo_gain: i32) {
            (**self).opposite_elo_gain_callback(elo_gain);
        }
    };
}

impl<T: Duel> Duel for &mut T {
    impl_duel!(@impl);
}

impl_duel!(Box);
