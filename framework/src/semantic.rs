//! 语义层定义 - Semantic Layer
//!
//! 定义语法到语义的映射：状态集合、转换函数、语义组合律

use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::collections::HashMap;

/// 语义单元类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SemanticType {
    State,
    Transition,
    Invariant,
    Action,
}

impl SemanticType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SemanticType::State => "state",
            SemanticType::Transition => "transition",
            SemanticType::Invariant => "invariant",
            SemanticType::Action => "action",
        }
    }
}

/// 语义单元 - 语义空间的基本元素
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SemanticUnit {
    pub semantic_type: SemanticType,
    pub name: String,
    pub properties: Vec<String>,
}

impl SemanticUnit {
    pub fn new(semantic_type: SemanticType, name: &str) -> Self {
        Self {
            semantic_type,
            name: name.to_string(),
            properties: Vec::new(),
        }
    }

    pub fn with_properties(mut self, props: Vec<String>) -> Self {
        self.properties = props;
        self
    }
}

/// 状态集合 + 转换函数 (语义表示)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateSet<S: Hash + Eq + Clone> {
    pub states: HashSet<S>,
    pub transitions: HashMap<S, HashSet<S>>,
}

impl<S: Hash + Eq + Clone> StateSet<S> {
    pub fn new(states: HashSet<S>) -> Self {
        let transitions = states.iter()
            .cloned()
            .map(|s| (s, HashSet::new()))
            .collect();
        Self { states, transitions }
    }

    pub fn with_transitions(mut self, transitions: HashMap<S, HashSet<S>>) -> Self {
        self.transitions = transitions;
        self
    }

    /// 语义组合: S(A) ∘ S(B) = S(A; B) 顺序执行
    /// 返回包含复合状态 (S, S) 的新 StateSet
    pub fn compose(&self, other: &StateSet<S>) -> StateSet<(S, S)> {
        let mut new_states: HashSet<(S, S)> = HashSet::new();
        let mut new_transitions: HashMap<(S, S), HashSet<(S, S)>> = HashMap::new();

        // 构造笛卡尔积状态空间
        for s1 in &self.states {
            for s2 in &other.states {
                new_states.insert((s1.clone(), s2.clone()));
            }
        }

        // 构造转换函数
        for (s1, s2) in &new_states {
            let mut next_states: HashSet<(S, S)> = HashSet::new();

            // 从 s1 的转换
            if let Some(trans) = self.transitions.get(s1) {
                for next_s1 in trans {
                    next_states.insert((next_s1.clone(), s2.clone()));
                }
            }

            // 从 s2 的转换
            if let Some(trans) = other.transitions.get(s2) {
                for next_s2 in trans {
                    next_states.insert((s1.clone(), next_s2.clone()));
                }
            }

            new_transitions.insert((s1.clone(), s2.clone()), next_states);
        }

        StateSet {
            states: new_states,
            transitions: new_transitions,
        }
    }

    /// 语义联合: S(A) ∪ S(B) = S(A + B) 并行/选择
    pub fn union(&self, other: &StateSet<S>) -> StateSet<S> {
        let new_states: HashSet<S> = self.states.union(&other.states).cloned().collect();

        let mut new_transitions: HashMap<S, HashSet<S>> = self.transitions.clone();

        for (s, trans) in &other.transitions {
            new_transitions
                .entry(s.clone())
                .and_modify(|t| *t = t.union(trans).cloned().collect())
                .or_insert_with(|| trans.clone());
        }

        StateSet {
            states: new_states,
            transitions: new_transitions,
        }
    }

    /// 验证两个语义表示是否等价
    pub fn equals(&self, other: &StateSet<S>) -> bool {
        self.states == other.states && self.transitions == other.transitions
    }
}

/// 语义映射 trait - 从语法到语义的转换
pub trait SemanticMapping<Expr, S>
where
    S: Hash + Eq + Clone,
{
    fn map(&self, expr: &Expr) -> StateSet<S>;
}

/// 简单的字符串状态语义映射实现
pub struct StringSemanticMapping;

impl StringSemanticMapping {
    /// 基本元素的语义
    pub fn primitive_semantics(name: &str) -> StateSet<String> {
        let states: HashSet<String> = [name.to_string()].into_iter().collect();
        let transitions: HashMap<String, HashSet<String>> = [
            (name.to_string(), [name.to_string()].into_iter().collect())
        ].into_iter().collect();
        StateSet { states, transitions }
    }

    /// 加法的语义: S(A + B) = S(A) ∪ S(B)
    pub fn add_semantics(left: &StateSet<String>, right: &StateSet<String>) -> StateSet<String> {
        left.union(right)
    }

    /// 序列的语义: S(A; B) = S(A) ∘ S(B)
    /// 返回包含复合状态的 StateSet
    pub fn seq_semantics(first: &StateSet<String>, second: &StateSet<String>) -> StateSet<(String, String)> {
        first.compose(second)
    }
}

/// 语义空间构造
pub fn create_semantic_space() -> Vec<SemanticUnit> {
    let mut space = Vec::new();

    // 状态类型
    for name in ["initial", "final", "error", "idle", "running", "paused"] {
        space.push(SemanticUnit::new(SemanticType::State, name));
    }

    // 不变量类型
    for prop in ["positive", "negative", "neutral"] {
        space.push(SemanticUnit::new(SemanticType::Invariant, &format!("state_{}", prop))
            .with_properties(vec![prop.to_string()]));
    }

    // 动作类型
    for action in ["validate", "compute", "transform", "check"] {
        space.push(SemanticUnit::new(SemanticType::Action, action));
    }

    space
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::{Primitive, Add, Seq};

    #[test]
    fn test_state_set_composition() {
        // 创建两个简单的状态集合
        let a = StringSemanticMapping::primitive_semantics("a");
        let b = StringSemanticMapping::primitive_semantics("b");

        // 组合: a; b
        let composed = a.compose(&b);
        assert!(!composed.states.is_empty());
    }

    #[test]
    fn test_state_set_union() {
        let a = StringSemanticMapping::primitive_semantics("a");
        let b = StringSemanticMapping::primitive_semantics("b");

        // 联合: a + b
        let union = a.union(&b);
        assert_eq!(union.states.len(), 2);
    }

    #[test]
    fn test_add_semantics() {
        let a = StringSemanticMapping::primitive_semantics("a");
        let b = StringSemanticMapping::primitive_semantics("b");

        let sem = StringSemanticMapping::add_semantics(&a, &b);
        assert_eq!(sem.states.len(), 2);
    }

    #[test]
    fn test_semantic_space() {
        let space = create_semantic_space();
        assert!(!space.is_empty());

        // 验证状态类型
        let states: Vec<_> = space.iter()
            .filter(|u| u.semantic_type == SemanticType::State)
            .collect();
        assert!(!states.is_empty());
    }
}
