/// Settings trait for singletons
pub trait State {
    /// Initialize the singleton
    fn set(initial_config: Self);
    /// Get the instance globally 
    fn get_global() -> Self; 
    /// Clear the settings
    fn clear();
    /// Check whether it is configured
    fn is_configured() -> bool;
}