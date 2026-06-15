use std::ops::{Deref, DerefMut};

/// 包装 `Option<Box<T>>`，提供类似 C 指针的行为
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Mbox<T>(Option<Box<T>>);

impl<T> Mbox<T> {
  /// 创建一个空指针（nil）
  pub fn nil() -> Self {
    Self(None)
  }

  /// 创建一个指向值的指针
  pub fn new(value: T) -> Self {
    Self(Some(Box::new(value)))
  }

  /// 判断是否为空（等同于 C 的 `ptr == NULL`）
  pub fn is_nil(&self) -> bool {
    self.0.is_none()
  }

  /// 获取内部 Option 的引用（更安全的访问方式）
  pub fn as_ref(&self) -> Option<&T> {
    self.0.as_ref().map(Box::as_ref)
  }

  /// 获取内部 Option 的可变引用
  pub fn as_mut(&mut self) -> Option<&mut T> {
    self.0.as_mut().map(Box::as_mut)
  }

  /// 取出内部值，如果为空则 panic
  pub fn unwrap(self) -> T {
    *self.0.expect("Mbox is nil")
  }
}

// 实现 Deref：允许直接访问 T 的字段（注意：如果 is_nil() 为 true 则会 panic）
impl<T> Deref for Mbox<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    self.0.as_ref().expect("Attempt to deref nil Mbox").as_ref()
  }
}

// 实现 DerefMut：允许修改 T 的字段（同样需要非空）
impl<T> DerefMut for Mbox<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self
      .0
      .as_mut()
      .expect("Attempt to deref_mut nil Mbox")
      .as_mut()
  }
}

// 方便从 Option<Box<T>> 转换
impl<T> From<Option<Box<T>>> for Mbox<T> {
  fn from(inner: Option<Box<T>>) -> Self {
    Self(inner)
  }
}

// 方便转换为 Option<Box<T>>
impl<T> From<Mbox<T>> for Option<Box<T>> {
  fn from(wrapper: Mbox<T>) -> Self {
    wrapper.0
  }
}
