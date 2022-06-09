use std::marker::PhantomData;

use crate::chunk::index::Index;
use crate::chunk::Value;

// Array trait

pub trait Array<T: Value, I>: Value {
  fn new(default: T) -> Self;

  fn index(&self, index: I) -> T;
  fn index_ref(&self, index: I) -> &T;
  fn index_mut(&mut self, index: I) -> &mut T;
  #[inline]
  fn set(&mut self, index: I, value: T) {
    *self.index_mut(index) = value;
  }

  fn len(&self) -> usize;
  fn contains(&self, index: I) -> bool;

  fn slice(&self) -> &[T];
  fn slice_mut(&mut self) -> &mut [T];
}

// Array implementation

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct ConstArray<T, I: Index, const LEN: usize> {
  array: [T; LEN],
  _phantom: PhantomData<I>,
}

impl<T: Value, I: Index, const LEN: usize> Array<T, I> for ConstArray<T, I, LEN> {
  #[inline]
  fn new(default: T) -> Self {
    let array = [default; LEN];
    Self { array, _phantom: PhantomData::default() }
  }

  #[inline]
  fn index(&self, index: I) -> T { self.array[index.into_usize()] }
  #[inline]
  fn index_ref(&self, index: I) -> &T { std::ops::Index::index(&self.array, index.into_usize()) }
  #[inline]
  fn index_mut(&mut self, index: I) -> &mut T { std::ops::IndexMut::index_mut(&mut self.array, index.into_usize()) }

  #[inline]
  fn len(&self) -> usize { LEN }
  #[inline]
  fn contains(&self, index: I) -> bool { index.into_usize() < LEN }

  #[inline]
  fn slice(&self) -> &[T] { &self.array }
  #[inline]
  fn slice_mut(&mut self) -> &mut [T] { &mut self.array }
}
