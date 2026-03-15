//! 语法层定义 - Syntax Layer
//!
//! 定义状态空间代数的语法结构：基本元素、加法、序列、Kleene闭包

use std::fmt;

/// 语法层表达式基 trait
pub trait Expr: fmt::Debug + Clone {}

/// 基本元素 (Primitive)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Primitive {
    pub name: String,
}

impl Primitive {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }
}

impl Expr for Primitive {}

/// 加法组合 (A + B) - 并行/选择
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Add<L, R> {
    pub left: L,
    pub right: R,
}

impl<L, R> Add<L, R> {
    pub fn new(left: L, right: R) -> Self {
        Self { left, right }
    }
}

/// 序列组合 (A; B) - 顺序执行
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Seq<L, R> {
    pub first: L,
    pub second: R,
}

impl<L, R> Seq<L, R> {
    pub fn new(first: L, second: R) -> Self {
        Self { first, second }
    }
}

/// Kleene 闭包 (A*) - 零次或多次重复
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Star<E> {
    pub expr: E,
}

impl<E> Star<E> {
    pub fn new(expr: E) -> Self {
        Self { expr }
    }
}

/// 表达式构造辅助函数
pub fn prim(name: &str) -> Primitive {
    Primitive::new(name)
}

pub fn add<L, R>(left: L, right: R) -> Add<L, R> {
    Add::new(left, right)
}

pub fn seq<L, R>(first: L, second: R) -> Seq<L, R> {
    Seq::new(first, second)
}

pub fn star<E>(expr: E) -> Star<E> {
    Star::new(expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive() {
        let p = Primitive::new("a");
        assert_eq!(p.name, "a");
    }

    #[test]
    fn test_expression_composition() {
        // a + b
        let expr = add(prim("a"), prim("b"));
        assert_eq!(expr.left.name, "a");
        assert_eq!(expr.right.name, "b");
    }

    #[test]
    fn test_nested_expression() {
        // (a + b); c
        let expr = seq(
            add(prim("a"), prim("b")),
            prim("c")
        );
        assert!(matches!(expr.first, Add { .. }));
        assert!(matches!(expr.second, Primitive { .. }));
    }
}
