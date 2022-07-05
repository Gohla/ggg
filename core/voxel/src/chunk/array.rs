use std::marker::PhantomData;
use std::ops::Range;

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

  fn as_slice(&self) -> &[T];
  fn as_slice_mut(&mut self) -> &mut [T];

  fn slice(&self, range: Range<I>) -> &[T];
  fn slice_mut(&mut self, range: Range<I>) -> &mut [T];
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
  fn as_slice(&self) -> &[T] { &self.array }
  #[inline]
  fn as_slice_mut(&mut self) -> &mut [T] { &mut self.array }

  #[inline]
  fn slice(&self, range: Range<I>) -> &[T] { &self.array[range.start.into_usize()..range.end.into_usize()] }
  #[inline]
  fn slice_mut(&mut self, range: Range<I>) -> &mut [T] { &mut self.array[range.start.into_usize()..range.end.into_usize()] }
}

// From: https://github.com/serde-rs/serde/issues/1937#issuecomment-812137971
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
