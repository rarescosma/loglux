use std::ops::Deref;

use num_traits::{AsPrimitive, One};

type Float = f64;

pub trait Bounded {
    type Inner;
    fn current(&self) -> Self::Inner;
    fn max(&self) -> Self::Inner;
}

pub struct Stepped<B> {
    bounded: B,
    num_steps: Float,
}

pub trait StepperExt {
    fn with_num_steps(self, num_steps: u32) -> Stepped<Self>
    where
        Self: Sized;
}

impl<B> StepperExt for B {
    fn with_num_steps(self, num_steps: u32) -> Stepped<B> {
        Stepped { bounded: self, num_steps: num_steps as Float }
    }
}

impl<B> Deref for Stepped<B> {
    type Target = B;

    fn deref(&self) -> &Self::Target { &self.bounded }
}

impl<B, N> Stepped<B>
where
    B: Bounded<Inner = N>,
    N: One + Ord + AsPrimitive<Float>,
    Float: AsPrimitive<N>,
{
    pub fn step_up(&self) -> N {
        let mut step = self.current_step();
        let mut new_b = self.current();

        while new_b <= self.current() {
            step += 1;
            new_b = self.brightness_at(step);
        }
        new_b.min(self.max())
    }

    pub fn step_down(&self) -> N {
        let mut step = self.current_step();
        let mut new_b = self.current();

        while new_b >= self.current() && step >= 0 {
            step -= 1;
            new_b = self.brightness_at(step);
        }
        new_b
    }

    fn current_step(&self) -> isize {
        (self.num_steps * self.current().max(N::one()).as_().log(self.max().as_())).round() as _
    }

    fn brightness_at(&self, step_no: isize) -> N {
        self.max().as_().powf(step_no as Float / self.num_steps).as_()
    }
}
