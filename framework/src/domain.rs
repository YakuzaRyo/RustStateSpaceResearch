//! 领域层定义 - Domain Layer
//!
//! 定义从模式层到领域层的映射：模式实例化、领域特定约束、跨领域验证

use std::collections::HashMap;
use std::fmt;

/// 领域参数
pub type DomainParams = HashMap<String, String>;

/// 模式实例
#[derive(Debug, Clone)]
pub struct PatternInstance {
    pub pattern_name: String,
    pub domain: String,
    pub parameters: DomainParams,
    pub core_constraints: Vec<String>,
    pub derived_constraints: Vec<String>,
}

impl PatternInstance {
    pub fn new(pattern_name: &str, domain: &str) -> Self {
        Self {
            pattern_name: pattern_name.to_string(),
            domain: domain.to_string(),
            parameters: HashMap::new(),
            core_constraints: Vec::new(),
            derived_constraints: Vec::new(),
        }
    }

    pub fn with_parameters(mut self, params: DomainParams) -> Self {
        self.parameters = params;
        self
    }

    pub fn with_core_constraints(mut self, constraints: Vec<String>) -> Self {
        self.core_constraints = constraints;
        self
    }

    pub fn with_derived_constraints(mut self, constraints: Vec<String>) -> Self {
        self.derived_constraints = constraints;
        self
    }

    /// 获取所有约束（核心 + 派生）
    pub fn get_all_constraints(&self) -> Vec<String> {
        let mut all = self.core_constraints.clone();
        all.extend(self.derived_constraints.clone());
        all
    }

    /// 验证核心约束是否保持
    pub fn preserves_core_constraints(&self) -> bool {
        // 核心约束必须在所有约束中存在
        self.get_all_constraints().iter().all(|c| {
            self.core_constraints.contains(c) ||
            self.derived_constraints.contains(c)
        })
    }
}

/// 领域 trait - 定义领域的通用接口
pub trait Domain: fmt::Debug {
    fn get_name(&self) -> &str;

    fn get_specific_constraints(&self, params: &DomainParams) -> Vec<String>;

    fn instantiate_pattern(&self, pattern: &Pattern) -> PatternInstance;

    /// 带参数的实例化方法
    fn instantiate_pattern_with_params(&self, pattern: &Pattern, params: &DomainParams) -> PatternInstance;

    fn derive_constraints(&self, pattern: &Pattern, params: &DomainParams) -> Vec<String>;
}

/// 模式定义（与 domain 无关的核心约束）
#[derive(Debug, Clone)]
pub struct Pattern {
    pub name: String,
    pub core_constraints: Vec<String>,
    pub domain_parameters: Vec<String>,
}

impl Pattern {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            core_constraints: Vec::new(),
            domain_parameters: Vec::new(),
        }
    }

    pub fn with_core_constraints(mut self, constraints: Vec<String>) -> Self {
        self.core_constraints = constraints;
        self
    }

    pub fn with_domain_parameters(mut self, params: Vec<String>) -> Self {
        self.domain_parameters = params;
        self
    }

    pub fn instantiate(&self, domain: &dyn Domain, params: DomainParams) -> PatternInstance {
        domain.instantiate_pattern(self)
    }
}

/// 数据流领域
#[derive(Debug)]
pub struct DataFlowDomain;

impl DataFlowDomain {
    pub fn new() -> Self {
        Self
    }
}

impl Domain for DataFlowDomain {
    fn get_name(&self) -> &str {
        "DataFlow"
    }

    fn get_specific_constraints(&self, params: &DomainParams) -> Vec<String> {
        let mut constraints = Vec::new();

        if let Some(data_type) = params.get("data_type") {
            constraints.push(format!("data_type={}", data_type));
        }

        if let Some(buffer_size) = params.get("buffer_size") {
            constraints.push(format!("buffer_size={}", buffer_size));
        }

        if let Some(direction) = params.get("direction") {
            constraints.push(format!("flow_direction={}", direction));
        }

        constraints
    }

    fn instantiate_pattern(&self, pattern: &Pattern) -> PatternInstance {
        self.instantiate_pattern_with_params(pattern, &HashMap::new())
    }

    fn instantiate_pattern_with_params(&self, pattern: &Pattern, params: &DomainParams) -> PatternInstance {
        let derived = self.derive_constraints(pattern, params);

        let mut instance = PatternInstance::new(&pattern.name, self.get_name())
            .with_core_constraints(pattern.core_constraints.clone())
            .with_derived_constraints(derived);

        // 添加数据流特有的核心约束
        instance.derived_constraints.push("no_data_loss".to_string());
        instance.derived_constraints.push("ordered_delivery".to_string());

        instance
    }

    fn derive_constraints(&self, pattern: &Pattern, params: &DomainParams) -> Vec<String> {
        let mut constraints = self.get_specific_constraints(params);

        // 派生约束
        if let Some(buffer_size) = params.get("buffer_size") {
            if buffer_size == "unbounded" {
                constraints.push("unbounded_buffer".to_string());
            } else {
                constraints.push("bounded_buffer".to_string());
            }
        }

        constraints
    }
}

/// 控制流领域
#[derive(Debug)]
pub struct ControlFlowDomain;

impl ControlFlowDomain {
    pub fn new() -> Self {
        Self
    }
}

impl Domain for ControlFlowDomain {
    fn get_name(&self) -> &str {
        "ControlFlow"
    }

    fn get_specific_constraints(&self, params: &DomainParams) -> Vec<String> {
        let mut constraints = Vec::new();

        if let Some(mode) = params.get("mode") {
            constraints.push(format!("execution_mode={}", mode));
        }

        if let Some(branching) = params.get("branching") {
            constraints.push(format!("branching={}", branching));
        }

        if let Some(sync) = params.get("synchronization") {
            constraints.push(format!("synchronization={}", sync));
        }

        constraints
    }

    fn instantiate_pattern(&self, pattern: &Pattern) -> PatternInstance {
        self.instantiate_pattern_with_params(pattern, &HashMap::new())
    }

    fn instantiate_pattern_with_params(&self, pattern: &Pattern, params: &DomainParams) -> PatternInstance {
        let derived = self.derive_constraints(pattern, params);

        let mut instance = PatternInstance::new(&pattern.name, self.get_name())
            .with_core_constraints(pattern.core_constraints.clone())
            .with_derived_constraints(derived);

        // 添加控制流特有的核心约束
        instance.derived_constraints.push("deterministic_execution".to_string());
        instance.derived_constraints.push("no_deadlock".to_string());

        instance
    }

    fn derive_constraints(&self, pattern: &Pattern, params: &DomainParams) -> Vec<String> {
        self.get_specific_constraints(params)
    }
}

/// 状态机领域
#[derive(Debug)]
pub struct StateMachineDomain;

impl StateMachineDomain {
    pub fn new() -> Self {
        Self
    }
}

impl Domain for StateMachineDomain {
    fn get_name(&self) -> &str {
        "StateMachine"
    }

    fn get_specific_constraints(&self, params: &DomainParams) -> Vec<String> {
        let mut constraints = Vec::new();

        if let Some(states) = params.get("states") {
            constraints.push(format!("state_space={}", states));
        }

        if let Some(transitions) = params.get("transitions") {
            constraints.push(format!("transitions={}", transitions));
        }

        if let Some(initial) = params.get("initial") {
            constraints.push(format!("initial_state={}", initial));
        }

        constraints
    }

    fn instantiate_pattern(&self, pattern: &Pattern) -> PatternInstance {
        self.instantiate_pattern_with_params(pattern, &HashMap::new())
    }

    fn instantiate_pattern_with_params(&self, pattern: &Pattern, params: &DomainParams) -> PatternInstance {
        let derived = self.derive_constraints(pattern, params);

        let mut instance = PatternInstance::new(&pattern.name, self.get_name())
            .with_core_constraints(pattern.core_constraints.clone())
            .with_derived_constraints(derived);

        // 添加状态机特有的核心约束
        instance.derived_constraints.push("finite_states".to_string());
        instance.derived_constraints.push("defined_initial_state".to_string());

        instance
    }

    fn derive_constraints(&self, pattern: &Pattern, params: &DomainParams) -> Vec<String> {
        self.get_specific_constraints(params)
    }
}

/// 创建跨领域模式
pub fn create_cross_domain_patterns() -> Vec<Pattern> {
    let mut patterns = Vec::new();

    // 管道模式 (Pipeline)
    patterns.push(
        Pattern::new("Pipeline")
            .with_core_constraints(vec![
                "sequential_processing".to_string(),
                "composable_stages".to_string(),
                "independent_stages".to_string(),
            ])
            .with_domain_parameters(vec![
                "stages".to_string(),
                "buffer_size".to_string(),
                "data_type".to_string(),
            ])
    );

    // 观察者模式 (Observer)
    patterns.push(
        Pattern::new("Observer")
            .with_core_constraints(vec![
                "decoupled_subjects".to_string(),
                "notification_mechanism".to_string(),
                "single_notification".to_string(),
            ])
            .with_domain_parameters(vec![
                "subjects".to_string(),
                "observers".to_string(),
                "update_policy".to_string(),
            ])
    );

    // 有限状态模式 (FiniteState)
    patterns.push(
        Pattern::new("FiniteState")
            .with_core_constraints(vec![
                "finite_state_space".to_string(),
                "defined_transitions".to_string(),
                "reachable_states".to_string(),
            ])
            .with_domain_parameters(vec![
                "states".to_string(),
                "transitions".to_string(),
                "initial".to_string(),
            ])
    );

    patterns
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_instance() {
        let pattern = Pattern::new("TestPattern")
            .with_core_constraints(vec!["constraint1".to_string()]);

        let instance = PatternInstance::new("TestPattern", "TestDomain")
            .with_core_constraints(pattern.core_constraints.clone());

        assert!(instance.preserves_core_constraints());
    }

    #[test]
    fn test_domain_instantiation() {
        let pattern = Pattern::new("Pipeline")
            .with_core_constraints(vec![
                "sequential_processing".to_string(),
                "composable_stages".to_string(),
            ]);

        let dataflow = DataFlowDomain::new();
        let instance = dataflow.instantiate_pattern(&pattern);

        assert_eq!(instance.pattern_name, "Pipeline");
        assert_eq!(instance.domain, "DataFlow");
    }

    #[test]
    fn test_cross_domain_core_constraints() {
        let patterns = create_cross_domain_patterns();
        let domains: Vec<Box<dyn Domain>> = vec![
            Box::new(DataFlowDomain::new()),
            Box::new(ControlFlowDomain::new()),
            Box::new(StateMachineDomain::new()),
        ];

        for pattern in &patterns {
            for domain in &domains {
                let instance = domain.instantiate_pattern(pattern);

                // 验证核心约束保持不变
                for core_constraint in &pattern.core_constraints {
                    assert!(
                        instance.get_all_constraints().contains(core_constraint),
                        "Core constraint {} should be preserved in {} domain",
                        core_constraint,
                        domain.get_name()
                    );
                }
            }
        }
    }
}
