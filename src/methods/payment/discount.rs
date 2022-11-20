#[derive(Debug)]
pub enum DiscountValue {
    Percentage(i128), Absolute(i128)
}