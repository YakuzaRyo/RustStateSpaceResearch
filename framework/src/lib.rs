// State Space Framework
// A Rust-based framework for type-safe state space representation

pub mod state {
    /// Core state representation
    pub trait State {
        fn is_valid(&self) -> bool;
    }
}

pub mod transition {
    /// State transition logic
    pub trait Transition<T> {
        fn can_transition(&self, to: &T) -> bool;
    }
}

pub mod validation {
    /// State validation utilities
    pub fn validate_state<T>(state: &T) -> bool
    where
        T: super::state::State,
    {
        state.is_valid()
    }
}
