type Num = u64;
type Float = f64;

pub trait Bounded {
    fn current(&self) -> Num;
    fn max(&self) -> Num;
    fn with_current(&self, current: Num) -> Self;
}

pub trait Stepper<B> {
    fn step_up(&self, num_steps: Num) -> Num;
    fn step_down(&self, num_steps: Num) -> Num;
}

impl<B: Bounded> Stepper<B> for B {
    fn step_up(&self, num_steps: Num) -> Num {
        if self.current() == 0 {
            return 1;
        }
        let mut step = current_step(self, num_steps);
        let mut new_b = self.current();

        while new_b <= self.current() {
            step += 1;
            new_b = brightness_at(self, step, num_steps);
        }

        let new_b = new_b.min(self.max());

        let lower = self.with_current(new_b).step_down(num_steps);
        if lower > self.current() && lower < new_b {
            return lower;
        }
        new_b
    }

    fn step_down(&self, num_steps: Num) -> Num {
        if self.current() == 1 {
            return 0;
        }
        let mut step = current_step(self, num_steps);
        let mut new_b = self.current();

        while new_b >= self.current() && step >= 1 {
            step -= 1;
            new_b = brightness_at(self, step, num_steps);
        }
        new_b
    }
}

fn current_step<B: Bounded>(b: &B, num_steps: Num) -> isize {
    (num_steps as Float * (b.current().max(1) as Float).log(b.max() as Float)).ceil() as _
}

fn brightness_at<B: Bounded>(b: &B, step_no: isize, num_steps: Num) -> Num {
    (b.max() as Float).powf(step_no as Float / num_steps as Float) as _
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use quickcheck::*;

    use super::*;

    type BaseNum = u16;

    const MAX_MAX: BaseNum = 2 << 12;

    #[derive(Copy, Clone, Debug)]
    struct DiffBase {
        base: BaseNum,
        diff: BaseNum,
        num_steps: BaseNum,
    }

    impl Arbitrary for DiffBase {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                diff: BaseNum::arbitrary(g),
                base: BaseNum::arbitrary(g),
                num_steps: BaseNum::arbitrary(g),
            }
        }
    }

    impl DiffBase {
        fn is_boring(&self) -> bool {
            !(1..MAX_MAX / 2).contains(&self.diff)
                || self.base > MAX_MAX / 2 - 1
                || self.num_steps < 1
        }
    }

    #[derive(Copy, Clone, Debug)]
    struct MockBounded {
        current: Num,
        max: Num,
    }

    impl Bounded for MockBounded {
        fn current(&self) -> Num { self.current }
        fn max(&self) -> Num { self.max }
        fn with_current(&self, current: Num) -> Self { Self { current, ..*self } }
    }

    impl From<&DiffBase> for MockBounded {
        fn from(db: &DiffBase) -> Self {
            MockBounded { current: db.base as Num, max: (db.base + db.diff) as Num }
        }
    }

    fn step_up_higher(db: DiffBase) -> TestResult {
        if db.is_boring() {
            return TestResult::discard();
        }

        let sut = MockBounded::from(&db);
        TestResult::from_bool(sut.step_up(db.num_steps as Num) > sut.current())
    }

    #[test]
    fn test_step_up_higher() { quickcheck(step_up_higher as fn(DiffBase) -> TestResult); }

    fn step_down_lower(db: DiffBase) -> TestResult {
        if db.is_boring() {
            return TestResult::discard();
        }

        let sut = MockBounded::from(&db);
        TestResult::from_bool(sut.step_down(db.num_steps as Num) <= sut.current())
    }

    #[test]
    fn test_step_down_lower() { quickcheck(step_down_lower as fn(DiffBase) -> TestResult); }

    fn step_invariantly(max: BaseNum, steps: BaseNum) -> TestResult {
        if !(1..=MAX_MAX).contains(&max) || !(1..=MAX_MAX).contains(&steps) {
            return TestResult::discard();
        }

        let max = max as Num;
        let steps = steps as Num;

        let mut sut = MockBounded { current: 0, max };

        let mut up_set = HashSet::new();
        up_set.insert(0);
        while sut.current < max {
            sut = sut.with_current(sut.step_up(steps));
            up_set.insert(sut.current);
        }

        let mut down_set = HashSet::new();
        down_set.insert(max);
        while sut.current > 0 {
            sut = sut.with_current(sut.step_down(steps));
            down_set.insert(sut.current);
        }

        TestResult::from_bool(up_set.eq(&down_set))
    }

    #[test]
    fn test_step_invariantly() {
        quickcheck(step_invariantly as fn(BaseNum, BaseNum) -> TestResult);
    }
}
