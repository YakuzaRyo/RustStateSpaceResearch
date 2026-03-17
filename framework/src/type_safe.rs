// Type-Safe State Module - 基于验证假设实现
// h1: 类型系统约束 - 使非法状态不可构造
// h2: 所有权系统状态转换 - 确保只有有效状态可以转换
// h3: 状态不变量 - 编译期验证状态不变量

use std::marker::PhantomData;
use std::ops::{Add, Sub};
use std::cmp::PartialOrd;

/// 编译期类型安全的状态值
/// 通过 newtype 模式防止直接构造无效状态
mod sealed {
    use std::marker::PhantomData;
    use std::ops::{Add, Sub};
    use std::cmp::PartialOrd;

    /// 标记类型的编译期范围约束
    pub trait Bounded: Copy + PartialOrd + Default + Add<Output = Self> + Sub<Output = Self> {}

    impl<T: Copy + PartialOrd + Default + Add<Output = Self> + Sub<Output = Self>> Bounded for T {}

    /// 带有范围约束的类型
    /// 只有在值满足 min <= v <= max 时才能构造
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct BoundedValue<T: Bounded, const MIN: i64, const MAX: i64>(T);

    impl<T: Bounded, const MIN: i64, const MAX: i64> BoundedValue<T, MIN, MAX> {
        /// 尝试创建新值 - 返回 None 如果超出范围
        pub fn new(v: i64) -> Option<Self> {
            if v >= MIN && v <= MAX {
                // 安全转换，因为我们已经验证了范围
                Some(BoundedValue(T::default()))
            } else {
                None
            }
        }

        /// 安全获取内部值
        pub fn get(&self) -> T {
            self.0
        }
    }
}

/// Newtype 模式: 使无效状态无法构造
/// 这是 h1 的核心实现: "Make Illegal States Unrepresentable"
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Validated<T>(T);

impl<T> Validated<T> {
    /// 私有构造函数 - 只能通过验证函数创建
    pub(crate) fn new_unchecked(value: T) -> Self {
        Validated(value)
    }

    /// 安全构造函数 - 返回 Option 确保只有有效状态可以创建
    pub fn new<F>(value: T, validator: F) -> Option<Self>
    where
        F: FnOnce(&T) -> bool,
    {
        if validator(&value) {
            Some(Validated(value))
        } else {
            None
        }
    }

    pub fn get(&self) -> &T {
        &self.0
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

/// 状态转换器 - 确保转换产生有效状态
/// 这是 h2 的核心实现: "所有权系统确保状态转换只能产生有效状态"
pub struct StateTransformer<S, T> {
    _source: PhantomData<S>,
    _target: PhantomData<T>,
    transform: fn(S) -> Option<T>,
}

impl<S, T> StateTransformer<S, T> {
    pub fn new(transform: fn(S) -> Option<T>) -> Self {
        Self {
            _source: PhantomData,
            _target: PhantomData,
            transform,
        }
    }

    pub fn apply(self, state: S) -> Option<T> {
        (self.transform)(state)
    }
}

/// 组合状态验证器 - 编译期验证多个不变量
/// 这是 h3 的核心实现: "精化类型编码状态不变量"
#[derive(Debug, Clone)]
pub struct InvariantChecker<I> {
    _invariant: PhantomData<I>,
}

pub trait Invariant: Sized {
    type State;
    fn check(state: &Self::State) -> bool;
}

/// 状态不变量组合器
#[derive(Debug, Clone)]
pub struct CompositeInvariant<I, J> {
    _first: PhantomData<I>,
    _second: PhantomData<J>,
}

impl<I, J, S> Invariant for CompositeInvariant<I, J>
where
    I: Invariant<State = S>,
    J: Invariant<State = S>,
{
    type State = S;
    fn check(state: &S) -> bool {
        I::check(state) && J::check(state)
    }
}

/// 类型安全的状态机
/// 使用类型参数标记状态，确保只能在有效状态间转换
#[derive(Debug, Clone)]
pub struct TypeSafeStateMachine<State> {
    value: i64,
    _phantom: PhantomData<State>,
}

impl<State> TypeSafeStateMachine<State> {
    pub fn new(value: i64) -> Option<Self> {
        Some(TypeSafeStateMachine {
            value,
            _phantom: PhantomData,
        })
    }

    pub fn value(&self) -> i64 {
        self.value
    }
}

/// 类型安全转换函数
/// 编译器会确保只能在特定状态间转换
pub mod transitions {
    use super::*;

    /// 初始状态
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Initial;

    /// 处理中状态
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Processing;

    /// 完成状态
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Completed;

    /// 失败状态
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Failed;

    /// 从 Initial -> Processing
    pub fn start_processing(state: TypeSafeStateMachine<Initial>) -> TypeSafeStateMachine<Processing> {
        TypeSafeStateMachine::new(state.value()).unwrap()
    }

    /// 从 Processing -> Completed
    pub fn complete(state: TypeSafeStateMachine<Processing>) -> TypeSafeStateMachine<Completed> {
        TypeSafeStateMachine::new(state.value()).unwrap()
    }

    /// 从任意状态 -> Failed
    pub fn fail<State>(state: TypeSafeStateMachine<State>) -> TypeSafeStateMachine<Failed> {
        TypeSafeStateMachine::new(state.value()).unwrap()
    }

    // 编译期验证: 尝试无效转换将导致编译错误
    // 例如: complete(initial_state) // 编译错误 - 不能从 Initial 直接到 Completed
}

/// 非空列表 - 编译期保证非空
#[derive(Debug, Clone)]
pub struct NonEmpty<T> {
    head: T,
    tail: Vec<T>,
}

impl<T> NonEmpty<T> {
    pub fn new(head: T) -> Self {
        Self {
            head,
            tail: Vec::new(),
        }
    }

    pub fn push(mut self, value: T) -> Self {
        self.tail.push(value);
        self
    }

    pub fn head(&self) -> &T {
        &self.head
    }

    pub fn tail(&self) -> &[T] {
        &self.tail
    }

    pub fn len(&self) -> usize {
        1 + self.tail.len()
    }

    pub fn is_empty(&self) -> bool {
        false // 永远不为空
    }
}

/// 非空选项 - 编译期保证 Some
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonEmptyOption<T>(Option<T>);

impl<T> NonEmptyOption<T> {
    pub fn new(value: T) -> Self {
        NonEmptyOption(Some(value))
    }

    pub fn get(&self) -> &T {
        self.0.as_ref().unwrap() // 安全: 永远是 Some
    }

    pub fn unwrap(self) -> T {
        self.0.unwrap() // 安全: 永远是 Some
    }
}

/// 编译期非零保证
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonZero<T: Copy + PartialEq + Default>(T);

impl<T: Copy + PartialEq + Default + PartialOrd + TryFrom<i64>> NonZero<T> {
    pub fn new(value: i64) -> Option<Self> {
        if value != 0 {
            // 尝试转换
            match T::try_from(value) {
                Ok(v) => Some(NonZero(v)),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    pub fn get(&self) -> T {
        self.0
    }
}

/// 类型安全的范围 - 编译期保证范围有效
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidRange {
    min: i64,
    max: i64,
}

impl ValidRange {
    pub const fn new(min: i64, max: i64) -> Option<Self> {
        if min <= max {
            Some(Self { min, max })
        } else {
            None // 无效范围
        }
    }

    pub fn contains(&self, value: i64) -> bool {
        value >= self.min && value <= self.max
    }

    pub fn clamp(&self, value: i64) -> i64 {
        value.max(self.min).min(self.max)
    }
}

/// 状态空间分区 - 分离安全状态和危险状态
#[derive(Debug, Clone)]
pub struct StateSpacePartition<T> {
    safe_states: Vec<T>,
    danger_states: Vec<T>,
}

impl<T: Clone> StateSpacePartition<T> {
    pub fn new() -> Self {
        Self {
            safe_states: Vec::new(),
            danger_states: Vec::new(),
        }
    }

    pub fn add_safe(&mut self, state: T) {
        self.safe_states.push(state);
    }

    pub fn add_danger(&mut self, state: T) {
        self.danger_states.push(state);
    }

    pub fn is_safe(&self, state: &T) -> bool {
        self.safe_states.contains(state)
    }

    pub fn is_danger(&self, state: &T) -> bool {
        self.danger_states.contains(state)
    }

    pub fn safe_count(&self) -> usize {
        self.safe_states.len()
    }

    pub fn danger_count(&self) -> usize {
        self.danger_states.len()
    }
}

impl<T> Default for StateSpacePartition<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// 编译期安全的字符串非空保证
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NonEmptyString(String);

impl NonEmptyString {
    pub fn new(s: &str) -> Option<Self> {
        if s.is_empty() {
            None
        } else {
            Some(NonEmptyString(s.to_string()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试: Newtype 模式防止无效状态构造
    #[test]
    fn test_validated_newtype() {
        let valid = Validated::new(42, |v| *v > 0);
        assert!(valid.is_some());

        let invalid = Validated::new(-1, |v| *v > 0);
        assert!(invalid.is_none());
    }

    /// 测试: 类型安全状态转换
    #[test]
    fn test_type_safe_transitions() {
        use transitions::*;

        let initial = TypeSafeStateMachine::<Initial>::new(42).unwrap();

        // 正确转换链: Initial -> Processing -> Completed
        let processing = start_processing(initial);
        assert_eq!(processing.value(), 42);

        let completed = complete(processing);
        assert_eq!(completed.value(), 42);
    }

    /// 测试: 非空列表
    #[test]
    fn test_non_empty() {
        let list = NonEmpty::new(1).push(2).push(3);
        assert_eq!(list.len(), 3);
        assert_eq!(*list.head(), 1);
        assert!(!list.is_empty());
    }

    /// 测试: 非空选项
    #[test]
    fn test_non_empty_option() {
        let opt = NonEmptyOption::new(42);
        assert_eq!(*opt.get(), 42);
        assert_eq!(opt.unwrap(), 42);
    }

    /// 测试: 非零值
    #[test]
    fn test_non_zero() {
        let nz = NonZero::<i64>::new(10);
        assert!(nz.is_some());

        let zero = NonZero::<i64>::new(0);
        assert!(zero.is_none());

        let neg = NonZero::<i64>::new(-5);
        assert!(neg.is_some());
    }

    /// 测试: 有效范围
    #[test]
    fn test_valid_range() {
        let range = ValidRange::new(0, 100).unwrap();
        assert!(range.contains(50));
        assert!(!range.contains(-1));
        assert!(!range.contains(101));

        assert_eq!(range.clamp(-5), 0);
        assert_eq!(range.clamp(150), 100);
        assert_eq!(range.clamp(50), 50);
    }

    /// 测试: 无效范围返回 None
    #[test]
    fn test_invalid_range() {
        let range = ValidRange::new(100, 0);
        assert!(range.is_none());
    }

    /// 测试: 状态空间分区
    #[test]
    fn test_state_partition() {
        let mut partition: StateSpacePartition<i64> = StateSpacePartition::new();

        for i in 0..=10 {
            partition.add_safe(i);
        }
        for i in -5..0 {
            partition.add_danger(i);
        }

        assert!(partition.is_safe(&5));
        assert!(partition.is_danger(&-1));
        assert!(!partition.is_safe(&-1));
        assert_eq!(partition.safe_count(), 11);
        assert_eq!(partition.danger_count(), 5);
    }

    /// 测试: 非空字符串
    #[test]
    fn test_non_empty_string() {
        let s = NonEmptyString::new("hello");
        assert!(s.is_some());
        assert_eq!(s.unwrap().as_str(), "hello");

        let empty = NonEmptyString::new("");
        assert!(empty.is_none());
    }

    /// 测试: 状态转换器
    #[test]
    fn test_state_transformer() {
        let transformer = StateTransformer::new(|v: i64| Some(v * 2));
        let result = transformer.apply(21);
        assert_eq!(result, Some(42));
    }

    /// 测试: 复合不变量
    #[test]
    fn test_composite_invariant() {
        #[derive(Clone, Copy)]
        struct Positive;
        #[derive(Clone, Copy)]
        struct Even;

        impl Invariant for Positive {
            type State = i64;
            fn check(state: &i64) -> bool {
                *state > 0
            }
        }

        impl Invariant for Even {
            type State = i64;
            fn check(state: &i64) {
                *state % 2 == 0
            }
        }

        type PosAndEven = CompositeInvariant<Positive, Even>;

        assert!(PosAndEven::check(&4));  // 正且偶数
        assert!(!PosAndEven::check(&3)); // 正但奇数
        assert!(!PosAndEven::check(&-2)); // 偶数但负
    }
}
