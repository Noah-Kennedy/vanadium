pub trait TransformIndependent<T> {
    type Output;
    fn sqrt(&self, out: &mut Self::Output);
    fn scale(&self, out: &mut Self::Output, n: T);
    fn add_to(&self, out: &mut Self::Output, n: T);
    fn ceiling(&self, out: &mut Self::Output);
    fn floor(&self, out: &mut Self::Output);
    fn clamp(&mut self, out: &mut Self::Output, floor: T, ceiling: T);
    fn map(&self, out: &mut Self::Output, f: &(dyn Fn(T) -> T + Send + Sync));
}

pub trait TransformIndependentInPlace<T> {
    fn sqrt_in_place(&mut self);
    fn scale_in_place(&mut self, n: T);
    fn add_in_place(&mut self, n: T);
    fn ceiling_in_place(&mut self);
    fn floor_in_place(&mut self);
    fn clamp_in_place(&mut self, floor: T, ceiling: T);
    fn map_in_place(&mut self, f: &(dyn Fn(T) -> T + Send + Sync));
}