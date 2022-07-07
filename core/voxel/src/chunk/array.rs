use std::marker::PhantomData;
use std::ops::{Range, RangeFull};

use crate::chunk::Value;

// Index trait + implementations

pub trait Index: Value {
  fn from_u32(i: u32) -> Self;
  fn into_u32(self) -> u32;
  fn into_usize(self) -> usize;
}

impl Index for u32 {
  #[inline]
  fn from_u32(i: u32) -> Self { i }
  #[inline]
  fn into_u32(self) -> u32 { self }
  #[inline]
  fn into_usize(self) -> usize { self as usize }
}


// Slice & Array trait

pub trait Slice<T: Value, I>: Value + std::ops::Index<I, Output=T> {
  fn len(&self) -> usize;
  fn contains(&self, index: I) -> bool;
}

pub trait SliceMut<T: Value, I>: Slice<T, I> + std::ops::IndexMut<I, Output=T> {
  #[inline]
  fn set(&mut self, index: I, value: T) {
    *self.index_mut(index) = value;
  }
}

pub trait Array<T: Value, I>: SliceMut<T, I>
+ std::ops::Index<Range<I>, Output=[T]>
+ std::ops::Index<RangeFull, Output=[T]>
+ std::ops::IndexMut<Range<I>, Output=[T]>
+ std::ops::IndexMut<RangeFull, Output=[T]>
{
  fn new(default: T) -> Self;
}


// Array indexing trait + implementations

pub trait ArrayIndex<T, I> {
  type Output: ?Sized;
  fn index(self, slice: &[T]) -> &Self::Output;
  fn index_mut(self, slice: &mut [T]) -> &mut Self::Output;
}

impl<T, I: Index> ArrayIndex<T, I> for I {
  type Output = T;
  #[inline]
  fn index(self, slice: &[T]) -> &Self::Output {
    &(*slice)[self.into_usize()]
  }
  #[inline]
  fn index_mut(self, slice: &mut [T]) -> &mut Self::Output {
    &mut (*slice)[self.into_usize()]
  }
}

impl<T, I: Index> ArrayIndex<T, I> for Range<I> {
  type Output = [T];
  #[inline]
  fn index(self, slice: &[T]) -> &Self::Output {
    &(*slice)[self.start.into_usize()..self.end.into_usize()]
  }
  #[inline]
  fn index_mut(self, slice: &mut [T]) -> &mut Self::Output {
    &mut (*slice)[self.start.into_usize()..self.end.into_usize()]
  }
}

impl<T, I: Index> ArrayIndex<T, I> for RangeFull {
  type Output = [T];
  #[inline]
  fn index(self, slice: &[T]) -> &Self::Output {
    &(*slice)[..]
  }
  #[inline]
  fn index_mut(self, slice: &mut [T]) -> &mut Self::Output {
    &mut (*slice)[..]
  }
}


// Array implementation

#[repr(transparent)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ConstArray<T, I: Index, const LEN: usize> {
  #[cfg_attr(feature = "serde", serde(bound(serialize = "T: serde::Serialize", deserialize = "T: serde::Deserialize<'de>"), with = "serde_arrays"))]
  array: [T; LEN],
  _phantom: PhantomData<I>,
}

impl<T: Value, I: Index, IR: ArrayIndex<T, I>, const LEN: usize> std::ops::Index<IR> for ConstArray<T, I, LEN> {
  type Output = IR::Output;
  #[inline]
  fn index(&self, index: IR) -> &Self::Output { index.index(&self.array) }
}

impl<T: Value, I: Index, IR: ArrayIndex<T, I>, const LEN: usize> std::ops::IndexMut<IR> for ConstArray<T, I, LEN> {
  #[inline]
  fn index_mut(&mut self, index: IR) -> &mut Self::Output { index.index_mut(&mut self.array) }
}

impl<T: Value, I: Index, const LEN: usize> Slice<T, I> for ConstArray<T, I, LEN> {
  #[inline]
  fn len(&self) -> usize { LEN }
  #[inline]
  fn contains(&self, index: I) -> bool { index.into_usize() < LEN }
}

impl<T: Value, I: Index, const LEN: usize> SliceMut<T, I> for ConstArray<T, I, LEN> {}

impl<T: Value, I: Index, const LEN: usize> Array<T, I> for ConstArray<T, I, LEN> {
  #[inline]
  fn new(default: T) -> Self {
    let array = [default; LEN];
    Self { array, _phantom: PhantomData::default() }
  }
}


// Array serialization. From: https://github.com/serde-rs/serde/issues/1937#issuecomment-812137971

#[cfg(feature = "serde")]
mod serde_arrays {
  use std::{convert::TryInto, marker::PhantomData};

  use serde::{
    de::{SeqAccess, Visitor},
    Deserialize,
    Deserializer, ser::SerializeTuple, Serialize, Serializer,
  };

  pub fn serialize<S: Serializer, T: Serialize, const N: usize>(data: &[T; N], ser: S, ) -> Result<S::Ok, S::Error> {
    let mut s = ser.serialize_tuple(N)?;
    for item in data {
      s.serialize_element(item)?;
    }
    s.end()
  }

  struct ArrayVisitor<T, const N: usize>(PhantomData<T>);

  impl<'de, T, const N: usize> Visitor<'de> for ArrayVisitor<T, N> where T: Deserialize<'de> {
    type Value = [T; N];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str(&format!("an array of length {}", N))
    }

    #[inline]
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
      // OPTO: can be optimized using MaybeUninit
      let mut data = Vec::with_capacity(N);
      for _ in 0..N {
        match (seq.next_element())? {
          Some(val) => data.push(val),
          None => return Err(serde::de::Error::invalid_length(N, &self)),
        }
      }
      match data.try_into() {
        Ok(arr) => Ok(arr),
        Err(_) => unreachable!(),
      }
    }
  }

  pub fn deserialize<'de, D, T, const N: usize>(deserializer: D) -> Result<[T; N], D::Error> where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
  {
    deserializer.deserialize_tuple(N, ArrayVisitor::<T, N>(PhantomData))
  }
}
