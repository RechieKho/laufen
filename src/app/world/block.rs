pub trait Block: Default + std::marker::Copy {
    fn default_filled() -> Self;
    fn is_empty(&self) -> bool;
}
