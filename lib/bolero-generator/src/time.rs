use crate::{
    bounded::{BoundExt, BoundedValue},
    Driver, TypeGenerator, TypeGeneratorWithParams, TypeValueGenerator, ValueGenerator,
};
use core::{ops::Range, time::Duration};

pub struct DurationGenerator<Seconds, Nanos> {
    seconds: Seconds,
    nanos: Nanos,
}

const VALID_NANOS_RANGE: Range<u32> = 0..1_000_000_000;

impl<Seconds, Nanos> DurationGenerator<Seconds, Nanos>
where
    Seconds: ValueGenerator<Output = u64>,
    Nanos: ValueGenerator<Output = u32>,
{
    pub fn seconds<NewS: ValueGenerator<Output = u64>>(
        self,
        seconds: NewS,
    ) -> DurationGenerator<NewS, Nanos> {
        DurationGenerator {
            seconds,
            nanos: self.nanos,
        }
    }

    pub fn map_seconds<NewS: ValueGenerator<Output = u64>, F: Fn(Seconds) -> NewS>(
        self,
        map: F,
    ) -> DurationGenerator<NewS, Nanos> {
        DurationGenerator {
            seconds: map(self.seconds),
            nanos: self.nanos,
        }
    }

    pub fn nanos<NewE: ValueGenerator<Output = u32>>(
        self,
        nanos: NewE,
    ) -> DurationGenerator<Seconds, NewE> {
        DurationGenerator {
            seconds: self.seconds,
            nanos,
        }
    }

    pub fn map_nanos<NewE: ValueGenerator<Output = u64>, F: Fn(Nanos) -> NewE>(
        self,
        map: F,
    ) -> DurationGenerator<Seconds, NewE> {
        DurationGenerator {
            seconds: self.seconds,
            nanos: map(self.nanos),
        }
    }
}

impl<Seconds, Nanos> ValueGenerator for DurationGenerator<Seconds, Nanos>
where
    Seconds: ValueGenerator<Output = u64>,
    Nanos: ValueGenerator<Output = u32>,
{
    type Output = Duration;

    fn generate<D: Driver>(&self, driver: &mut D) -> Option<Self::Output> {
        driver.enter_product::<Duration, _, _>(|driver| {
            let seconds = self.seconds.generate(driver);
            let nanos = self.nanos.generate(driver);
            Some(Duration::new(seconds?, nanos?))
        })
    }

    fn mutate<D: Driver>(&self, driver: &mut D, value: &mut Duration) -> Option<()> {
        driver.enter_product::<Duration, _, _>(|driver| {
            let mut seconds = value.as_secs();
            self.seconds.mutate(driver, &mut seconds)?;
            let mut nanos = value.subsec_nanos();
            self.nanos.mutate(driver, &mut nanos)?;
            *value = Duration::new(seconds, nanos);
            Some(())
        })
    }
}

impl TypeGenerator for Duration {
    fn generate<D: Driver>(driver: &mut D) -> Option<Self> {
        Self::gen_with().generate(driver)
    }

    fn mutate<D: Driver>(&mut self, driver: &mut D) -> Option<()> {
        Self::gen_with().mutate(driver, self)
    }
}

impl TypeGeneratorWithParams for Duration {
    type Output = DurationGenerator<TypeValueGenerator<u64>, Range<u32>>;

    fn gen_with() -> Self::Output {
        DurationGenerator {
            seconds: Default::default(),
            nanos: VALID_NANOS_RANGE,
        }
    }
}

impl BoundedValue for Duration {
    #[inline]
    fn gen_bounded<D: Driver>(
        driver: &mut D,
        min: core::ops::Bound<&Self>,
        max: core::ops::Bound<&Self>,
    ) -> Option<Self> {
        let min = BoundExt::map(min, |min| min.as_nanos());
        let max = BoundExt::map(max, |max| max.as_nanos());
        let value = u128::gen_bounded(driver, BoundExt::as_ref(&min), BoundExt::as_ref(&max))?;
        let value = value.try_into().ok()?;
        let value = Duration::from_nanos(value);
        Some(value)
    }
}

#[test]
fn duration_test() {
    let _ = generator_test!(produce::<Duration>());
}
