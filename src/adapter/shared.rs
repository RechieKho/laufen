pub type Shared<T> = std::sync::Arc<std::sync::Mutex<T>>;
pub fn share<T>(p_value: T) -> Shared<T> {
    std::sync::Arc::new(std::sync::Mutex::new(p_value))
}
