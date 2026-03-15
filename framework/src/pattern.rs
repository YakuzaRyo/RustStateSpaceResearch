//! 模式层定义 - Pattern Layer
//!
//! 定义从语义层到模式层的映射：模式定义、模式匹配、覆盖率验证

use crate::semantic::{SemanticUnit, SemanticType};
use std::collections::HashSet;

/// 匹配条件
#[derive(Debug, Clone)]
pub enum MatchCondition {
    Type(String),           // 类型匹配
    NamePrefix(String),    // 名称前缀匹配
    Property(String),      // 属性匹配
}

/// 模式定义
#[derive(Debug, Clone)]
pub struct Pattern {
    pub name: String,
    pub match_conditions: Vec<MatchCondition>,
    pub covers: HashSet<SemanticUnit>,
}

impl Pattern {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            match_conditions: Vec::new(),
            covers: HashSet::new(),
        }
    }

    pub fn with_conditions(mut self, conditions: Vec<MatchCondition>) -> Self {
        self.match_conditions = conditions;
        self
    }

    /// 检查语义单元是否匹配此模式
    pub fn matches(&self, unit: &SemanticUnit) -> bool {
        if self.match_conditions.is_empty() {
            return true;
        }

        for condition in &self.match_conditions {
            match condition {
                MatchCondition::Type(type_name) => {
                    if unit.semantic_type.as_str() != *type_name {
                        return false;
                    }
                }
                MatchCondition::NamePrefix(prefix) => {
                    if !unit.name.starts_with(prefix) {
                        return false;
                    }
                }
                MatchCondition::Property(prop) => {
                    if !unit.properties.contains(prop) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

/// 模式集合创建
pub fn create_pattern_set() -> Vec<Pattern> {
    let mut patterns = Vec::new();

    // 模式1: 状态模式 - 匹配所有状态类型
    patterns.push(
        Pattern::new("StatePattern")
            .with_conditions(vec![MatchCondition::Type("state".to_string())])
    );

    // 模式2: 转换模式 - 匹配所有转换类型
    patterns.push(
        Pattern::new("TransitionPattern")
            .with_conditions(vec![MatchCondition::Type("transition".to_string())])
    );

    // 模式3: 不变量模式
    patterns.push(
        Pattern::new("InvariantPattern")
            .with_conditions(vec![MatchCondition::Type("invariant".to_string())])
    );

    // 模式4: 动作模式
    patterns.push(
        Pattern::new("ActionPattern")
            .with_conditions(vec![MatchCondition::Type("action".to_string())])
    );

    // 模式5: 初始状态模式
    patterns.push(
        Pattern::new("InitialStatePattern")
            .with_conditions(vec![
                MatchCondition::Type("state".to_string()),
                MatchCondition::NamePrefix("initial".to_string()),
            ])
    );

    // 模式6: 错误处理模式
    patterns.push(
        Pattern::new("ErrorPattern")
            .with_conditions(vec![MatchCondition::NamePrefix("error".to_string())])
    );

    patterns
}

/// 覆盖率结果
#[derive(Debug)]
pub struct CoverageResult {
    pub total_units: usize,
    pub covered_units: HashSet<SemanticUnit>,
    pub uncovered_units: HashSet<SemanticUnit>,
    pub pattern_coverage: Vec<(String, usize)>,
    pub conflicts: Vec<(String, String, String)>,
}

/// 计算模式对语义空间的覆盖率
pub fn compute_coverage(patterns: &[Pattern], semantic_space: &[SemanticUnit]) -> CoverageResult {
    let mut covered_units: HashSet<SemanticUnit> = HashSet::new();
    let mut uncovered_units: HashSet<SemanticUnit> = HashSet::new();
    let mut pattern_coverage: Vec<(String, usize)> = Vec::new();
    let mut conflicts: Vec<(String, String, String)> = Vec::new();

    for unit in semantic_space {
        let matched_patterns: Vec<_> = patterns.iter()
            .filter(|p| p.matches(unit))
            .collect();

        if matched_patterns.is_empty() {
            uncovered_units.insert(unit.clone());
        } else {
            covered_units.insert(unit.clone());

            for p in &matched_patterns {
                pattern_coverage.push((p.name.clone(), 1));
            }

            // 检查冲突（多个模式匹配同一单元）
            if matched_patterns.len() > 1 {
                for i in 0..matched_patterns.len() {
                    for j in (i + 1)..matched_patterns.len() {
                        conflicts.push((
                            matched_patterns[i].name.clone(),
                            matched_patterns[j].name.clone(),
                            unit.name.clone(),
                        ));
                    }
                }
            }
        }
    }

    // 合并相同模式的覆盖率计数
    let mut merged_coverage: Vec<(String, usize)> = Vec::new();
    let mut coverage_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (name, count) in pattern_coverage {
        *coverage_map.entry(name).or_insert(0) += count;
    }
    for (name, count) in coverage_map {
        merged_coverage.push((name, count));
    }

    CoverageResult {
        total_units: semantic_space.len(),
        covered_units,
        uncovered_units,
        pattern_coverage: merged_coverage,
        conflicts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{SemanticType, create_semantic_space};

    #[test]
    fn test_pattern_matching() {
        let state_pattern = Pattern::new("StatePattern")
            .with_conditions(vec![MatchCondition::Type("state".to_string())]);

        let state_unit = SemanticUnit::new(SemanticType::State, "idle");
        let action_unit = SemanticUnit::new(SemanticType::Action, "validate");

        assert!(state_pattern.matches(&state_unit));
        assert!(!state_pattern.matches(&action_unit));
    }

    #[test]
    fn test_pattern_coverage() {
        let patterns = create_pattern_set();
        let semantic_space = create_semantic_space();

        let coverage = compute_coverage(&patterns, &semantic_space);

        println!("Total units: {}", coverage.total_units);
        println!("Covered: {}", coverage.covered_units.len());
        println!("Uncovered: {}", coverage.uncovered_units.len());

        // 验证覆盖率
        let coverage_rate = coverage.covered_units.len() as f64 / coverage.total_units as f64;
        assert!(coverage_rate >= 1.0, "Coverage should be 100%");
    }

    #[test]
    fn test_finite_patterns() {
        let patterns = create_pattern_set();
        assert!(!patterns.is_empty());
        assert!(patterns.len() < usize::MAX);
    }
}
