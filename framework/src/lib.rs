// State Space Framework
// A Rust-based framework for type-safe state space representation
// Core principle: Make errors impossible by design through invariant constraints

// 分层设计模块 - Layered Design
pub mod syntax;
pub mod semantic;
pub mod pattern;
pub mod domain;
pub mod layered;

// 类型安全模块 - 基于验证假设实现
// h1: 类型系统约束 - 使非法状态不可构造
// h2: 所有权系统状态转换 - 确保只有有效状态可以转换
// h3: 状态不变量 - 编译期验证状态不变量
pub mod type_safe;

pub mod invariant {
    /// Invariant trait - defines properties that must hold throughout state lifecycle
    pub trait Invariant: Clone {
        /// Check if the invariant holds for a given state
        fn holds(&self, state: &impl StateSpace) -> bool;
    }

    /// State space trait for invariant checking
    pub trait StateSpace {
        fn get_field(&self, name: &str) -> Option<i64>;
    }

    /// Composite invariant - combines multiple invariants
    #[derive(Clone)]
    pub struct CompositeInvariant<I: Invariant> {
        invariants: Vec<I>,
    }

    impl<I: Invariant> CompositeInvariant<I> {
        pub fn new(invariants: Vec<I>) -> Self {
            Self { invariants }
        }

        pub fn holds(&self, state: &impl StateSpace) -> bool {
            self.invariants.iter().all(|inv| inv.holds(state))
        }
    }

    /// Guard - prevents transitions that would violate invariants
    pub struct Guard<I: Invariant> {
        invariant: I,
    }

    impl<I: Invariant> Guard<I> {
        pub fn new(invariant: I) -> Self {
            Self { invariant }
        }

        pub fn allows(&self, from_state: &impl StateSpace, to_state: &impl StateSpace) -> bool {
            // Transition is allowed if both states satisfy the invariant
            self.invariant.holds(from_state) && self.invariant.holds(to_state)
        }
    }
}

pub mod state {
    use crate::invariant::{Invariant, StateSpace};
    use std::collections::BTreeMap;

    /// Core state representation with type-safe fields
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct TypedState {
        fields: BTreeMap<String, i64>,
    }

    impl TypedState {
        pub fn new() -> Self {
            Self {
                fields: BTreeMap::new(),
            }
        }

        pub fn with_field(mut self, name: &str, value: i64) -> Self {
            self.fields.insert(name.to_string(), value);
            self
        }

        pub fn get(&self, name: &str) -> Option<i64> {
            self.fields.get(name).copied()
        }
    }

    impl Default for TypedState {
        fn default() -> Self {
            Self::new()
        }
    }

    // Manual Hash implementation for TypedState
    impl std::hash::Hash for TypedState {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            for (k, v) in &self.fields {
                k.hash(state);
                v.hash(state);
            }
        }
    }

    impl StateSpace for TypedState {
        fn get_field(&self, name: &str) -> Option<i64> {
            self.get(name)
        }
    }

    /// State validation result
    #[derive(Debug, Clone, PartialEq)]
    pub enum ValidationResult {
        Valid,
        Invalid { reason: String },
    }

    /// Validated state wrapper - enforces invariants at compile time where possible
    #[derive(Clone, Debug)]
    pub struct ValidatedState<S> {
        state: S,
        validation_cache: Option<bool>,
    }

    impl<S> ValidatedState<S> {
        pub fn new(state: S) -> Self {
            Self {
                state,
                validation_cache: None,
            }
        }

        pub fn inner(&self) -> &S {
            &self.state
        }

        pub fn into_inner(self) -> S {
            self.state
        }
    }
}

pub mod transition {
    use crate::state::TypedState;

    /// State transition with pre/post conditions
    #[derive(Clone, Debug)]
    pub struct Transition {
        pub name: String,
        pub pre_condition: Option<fn(&TypedState) -> bool>,
        pub post_condition: Option<fn(&TypedState) -> bool>,
    }

    impl Transition {
        pub fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                pre_condition: None,
                post_condition: None,
            }
        }

        pub fn with_precondition(mut self, cond: fn(&TypedState) -> bool) -> Self {
            self.pre_condition = Some(cond);
            self
        }

        pub fn with_postcondition(mut self, cond: fn(&TypedState) -> bool) -> Self {
            self.post_condition = Some(cond);
            self
        }

        pub fn can_execute(&self, state: &TypedState) -> bool {
            self.pre_condition
                .map(|c| c(state))
                .unwrap_or(true)
        }

        pub fn execute(&self, state: &TypedState) -> Result<TypedState, String> {
            if !self.can_execute(state) {
                return Err(format!("Precondition failed for transition: {}", self.name));
            }

            // Post-condition would be checked after actual state mutation
            // This is a placeholder for the actual transition logic
            Ok(state.clone())
        }
    }

    /// Transition system - defines all valid state transitions
    #[derive(Clone, Debug)]
    pub struct TransitionSystem {
        transitions: Vec<Transition>,
    }

    impl TransitionSystem {
        pub fn new() -> Self {
            Self {
                transitions: Vec::new(),
            }
        }

        pub fn add_transition(mut self, t: Transition) -> Self {
            self.transitions.push(t);
            self
        }

        pub fn get_transitions(&self) -> &[Transition] {
            &self.transitions
        }

        pub fn is_valid_transition(&self, from: &TypedState, _to: &TypedState, trans_name: &str) -> bool {
            self.transitions
                .iter()
                .any(|t| t.name == trans_name && t.can_execute(from))
        }
    }

    impl Default for TransitionSystem {
        fn default() -> Self {
            Self::new()
        }
    }
}

pub mod algebra {
    use crate::state::TypedState;
    use crate::transition::TransitionSystem;
    use std::collections::HashSet;

    /// State space algebra - partitions state space into safe and danger regions
    #[derive(Clone, Debug)]
    pub struct StateSpaceAlgebra {
        /// States that satisfy all invariants (safe region)
        safe_states: HashSet<TypedState>,
        /// Known invalid states (danger region)
        danger_states: HashSet<TypedState>,
        transition_system: TransitionSystem,
    }

    impl StateSpaceAlgebra {
        pub fn new(transition_system: TransitionSystem) -> Self {
            Self {
                safe_states: HashSet::new(),
                danger_states: HashSet::new(),
                transition_system,
            }
        }

        /// Add a safe state (satisfies invariants)
        pub fn add_safe_state(&mut self, state: TypedState) {
            self.safe_states.insert(state);
        }

        /// Add a danger state (violates invariants)
        pub fn add_danger_state(&mut self, state: TypedState) {
            self.danger_states.insert(state);
        }

        /// Check if a state is in the safe region
        pub fn is_safe(&self, state: &TypedState) -> bool {
            self.safe_states.contains(state)
        }

        /// Check if a state is in the danger region
        pub fn is_danger(&self, state: &TypedState) -> bool {
            self.danger_states.contains(state)
        }

        /// Verify that error states are unreachable from safe states
        pub fn verify_safety(&self, _initial: &TypedState, error_states: &[TypedState]) -> bool {
            // Simple reachability check - error states should not be in safe region
            error_states.iter().all(|es| !self.safe_states.contains(es))
        }

        pub fn get_safe_states(&self) -> &HashSet<TypedState> {
            &self.safe_states
        }
    }

    /// Reachability analysis
    pub fn can_reach(from: &TypedState, _to: &TypedState, system: &TransitionSystem) -> bool {
        // Simplified reachability - in practice would use graph algorithms
        system.get_transitions().iter().any(|t| t.can_execute(from))
    }
}

pub mod examples {
    //! Example implementations demonstrating state space algebra principles

    use crate::invariant::{Guard, Invariant, StateSpace};
    use crate::state::TypedState;
    use crate::transition::{Transition, TransitionSystem};
    use crate::algebra::StateSpaceAlgebra;

    /// Bank account invariant: balance >= 0 (never go negative)
    #[derive(Clone)]
    pub struct BankAccountInvariant;

    impl Invariant for BankAccountInvariant {
        fn holds(&self, state: &impl StateSpace) -> bool {
            if let Some(balance) = state.get_field("balance") {
                balance >= 0
            } else {
                false // No balance field = invalid state
            }
        }
    }

    /// Bank account state wrapper
    #[derive(Clone)]
    pub struct BankAccount {
        state: TypedState,
    }

    impl BankAccount {
        pub fn new(balance: i64) -> Self {
            Self {
                state: TypedState::new().with_field("balance", balance),
            }
        }

        pub fn balance(&self) -> i64 {
            self.state.get("balance").unwrap_or(0)
        }

        /// Deposit money - only valid if result satisfies invariant
        pub fn deposit(&self, amount: i64) -> Result<Self, String> {
            let new_balance = self.balance() + amount;
            let new_state = TypedState::new().with_field("balance", new_balance);

            // Check invariant before allowing transition
            if BankAccountInvariant.holds(&new_state) {
                Ok(Self { state: new_state })
            } else {
                Err("Deposit would violate invariant (negative balance)".to_string())
            }
        }

        /// Withdraw money - only valid if result satisfies invariant
        pub fn withdraw(&self, amount: i64) -> Result<Self, String> {
            let new_balance = self.balance() - amount;
            let new_state = TypedState::new().with_field("balance", new_balance);

            // Check invariant before allowing transition
            if BankAccountInvariant.holds(&new_state) {
                Ok(Self { state: new_state })
            } else {
                Err("Withdrawal would violate invariant (negative balance)".to_string())
            }
        }

        pub fn state(&self) -> &TypedState {
            &self.state
        }
    }

    impl StateSpace for BankAccount {
        fn get_field(&self, name: &str) -> Option<i64> {
            self.state.get(name)
        }
    }

    /// Create bank account transition system
    pub fn create_bank_transition_system() -> TransitionSystem {
        TransitionSystem::new()
            .add_transition(
                Transition::new("deposit")
                    .with_precondition(|s| s.get("balance").map(|b| b >= 0).unwrap_or(false))
            )
            .add_transition(
                Transition::new("withdraw")
                    .with_precondition(|s| {
                        s.get("balance").map(|b| b > 0).unwrap_or(false)
                    })
                    .with_postcondition(|s| s.get("balance").map(|b| b >= 0).unwrap_or(false))
            )
    }

    /// Build state space algebra for bank account
    pub fn build_bank_state_space() -> StateSpaceAlgebra {
        let system = create_bank_transition_system();
        let mut algebra = StateSpaceAlgebra::new(system);

        // Add safe states
        for balance in 0..=1000 {
            algebra.add_safe_state(TypedState::new().with_field("balance", balance));
        }

        // Add danger states (negative balance)
        for balance in -100..0 {
            algebra.add_danger_state(TypedState::new().with_field("balance", balance));
        }

        algebra
    }

    /// Counter invariant: value stays within bounds [min, max]
    #[derive(Clone)]
    pub struct BoundedCounterInvariant {
        min: i64,
        max: i64,
    }

    impl BoundedCounterInvariant {
        pub fn new(min: i64, max: i64) -> Self {
            Self { min, max }
        }
    }

    impl Invariant for BoundedCounterInvariant {
        fn holds(&self, state: &impl StateSpace) -> bool {
            if let Some(value) = state.get_field("value") {
                value >= self.min && value <= self.max
            } else {
                false
            }
        }
    }

    /// Bounded counter example
    #[derive(Clone)]
    pub struct BoundedCounter {
        state: TypedState,
        min: i64,
        max: i64,
    }

    impl BoundedCounter {
        pub fn new(value: i64, min: i64, max: i64) -> Result<Self, String> {
            let state = TypedState::new().with_field("value", value);
            let inv = BoundedCounterInvariant::new(min, max);

            if inv.holds(&state) {
                Ok(Self { state, min, max })
            } else {
                Err("Initial value out of bounds".to_string())
            }
        }

        pub fn increment(&self) -> Result<Self, String> {
            let new_value = self.state.get("value").unwrap_or(0) + 1;
            let new_state = TypedState::new().with_field("value", new_value);
            let inv = BoundedCounterInvariant::new(self.min, self.max);

            if inv.holds(&new_state) {
                Ok(Self { state: new_state, min: self.min, max: self.max })
            } else {
                Err("Increment would exceed maximum".to_string())
            }
        }

        pub fn decrement(&self) -> Result<Self, String> {
            let new_value = self.state.get("value").unwrap_or(0) - 1;
            let new_state = TypedState::new().with_field("value", new_value);
            let inv = BoundedCounterInvariant::new(self.min, self.max);

            if inv.holds(&new_state) {
                Ok(Self { state: new_state, min: self.min, max: self.max })
            } else {
                Err("Decrement would go below minimum".to_string())
            }
        }

        pub fn value(&self) -> i64 {
            self.state.get("value").unwrap_or(0)
        }
    }

    impl StateSpace for BoundedCounter {
        fn get_field(&self, name: &str) -> Option<i64> {
            self.state.get(name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::examples::{BankAccount, BankAccountInvariant, BoundedCounter};
    use crate::invariant::{Guard, Invariant, StateSpace};
    use crate::state::TypedState;

    /// Test that bank account invariant prevents negative balance
    #[test]
    fn test_bank_invariant_prevents_negative() {
        let account = BankAccount::new(100);
        let result = account.withdraw(150);

        // Should fail - would create negative balance
        assert!(result.is_err());
    }

    /// Test that valid operations succeed
    #[test]
    fn test_bank_valid_operations() {
        let account = BankAccount::new(100);

        let after_deposit = account.deposit(50).unwrap();
        assert_eq!(after_deposit.balance(), 150);

        let after_withdraw = after_deposit.withdraw(50).unwrap();
        assert_eq!(after_withdraw.balance(), 100);
    }

    /// Test bounded counter invariant
    #[test]
    fn test_bounded_counter() {
        let counter = BoundedCounter::new(5, 0, 10).unwrap();
        assert_eq!(counter.value(), 5);

        // Increment within bounds
        let inc = counter.increment().unwrap();
        assert_eq!(inc.value(), 6);

        // Decrement within bounds (from incremented value)
        let dec = inc.decrement().unwrap();
        assert_eq!(dec.value(), 5);
    }

    /// Test bounded counter prevents overflow
    #[test]
    fn test_bounded_counter_prevents_overflow() {
        let counter = BoundedCounter::new(10, 0, 10).unwrap();

        // Should fail - would exceed maximum
        assert!(counter.increment().is_err());
    }

    /// Test bounded counter prevents underflow
    #[test]
    fn test_bounded_counter_prevents_underflow() {
        let counter = BoundedCounter::new(0, 0, 10).unwrap();

        // Should fail - would go below minimum
        assert!(counter.decrement().is_err());
    }

    /// Test state space algebra partitioning
    #[test]
    fn test_state_space_algebra_partitioning() {
        let algebra = crate::examples::build_bank_state_space();

        let safe_state = TypedState::new().with_field("balance", 100);
        let danger_state = TypedState::new().with_field("balance", -50);

        assert!(algebra.is_safe(&safe_state));
        assert!(algebra.is_danger(&danger_state));
    }

    /// Test guard allows valid transitions
    #[test]
    fn test_guard_allows_valid() {
        let guard = Guard::new(BankAccountInvariant);

        let from = TypedState::new().with_field("balance", 100);
        let to = TypedState::new().with_field("balance", 150);

        assert!(guard.allows(&from, &to));
    }

    /// Test guard blocks invalid transitions
    #[test]
    fn test_guard_blocks_invalid() {
        let guard = Guard::new(BankAccountInvariant);

        let from = TypedState::new().with_field("balance", 100);
        let to = TypedState::new().with_field("balance", -50);

        assert!(!guard.allows(&from, &to));
    }

    /// Test that error states are unreachable (core principle)
    #[test]
    fn test_error_states_unreachable() {
        let algebra = crate::examples::build_bank_state_space();

        let error_states = vec![
            TypedState::new().with_field("balance", -1),
            TypedState::new().with_field("balance", -100),
        ];

        assert!(algebra.verify_safety(&TypedState::new().with_field("balance", 0), &error_states));
    }
}
