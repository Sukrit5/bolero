use crate::{driver::Driver, TypeGenerator, TypeGeneratorWithParams, ValueGenerator};

#[derive(Debug)]
pub struct BooleanGenerator {
    weight: f32,
}

impl Default for BooleanGenerator {
    fn default() -> Self {
        Self { weight: 0.5 }
    }
}

impl ValueGenerator for bool {
    type Output = bool;

    fn generate<D: Driver>(&self, _driver: &mut D) -> Option<Self::Output> {
        Some(*self)
    }
}

impl BooleanGenerator {
    pub fn weight(mut self, weight: f32) -> Self {
        assert!((0.0..=1.0).contains(&weight));
        self.weight = weight;
        self
    }
}

impl TypeGeneratorWithParams for bool {
    type Output = BooleanGenerator;

    fn gen_with() -> Self::Output {
        Default::default()
    }
}

impl ValueGenerator for BooleanGenerator {
    type Output = bool;

    fn generate<D: Driver>(&self, driver: &mut D) -> Option<Self::Output> {
        driver.gen_bool(Some(self.weight))
    }
}

impl TypeGenerator for bool {
    fn generate<D: Driver>(driver: &mut D) -> Option<Self> {
        driver.gen_bool(None)
    }
}

#[test]
fn bool_test() {
    let _ = generator_test!(produce::<bool>());
}

#[test]
fn bool_with_test() {
    let _ = generator_test!(produce::<bool>().with().weight(0.1));
}
