/// Empty list.
#[repr(transparent)]
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Nil;

/// List constructor.
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Cons<Head, Tail> {
  head: Head,
  tail: Tail,
}
impl<Head, Tail> Cons<Head, Tail> {
  #[inline]
  pub fn new(head: Head, tail: Tail) -> Self { Self { head, tail } }
}

/// List trait
pub trait List: sealed::Sealed {}
impl List for Nil {}
impl<Head, Tail: List> List for Cons<Head, Tail> {}
mod sealed {
  pub trait Sealed {}
  impl Sealed for super::Nil {}
  impl<Head, Tail> Sealed for super::Cons<Head, Tail> where Tail: Sealed {}
}

/// Prepend to list
pub trait Prepend: List {
  type Output<T>: List;
  fn prepend<T>(self, value: T) -> Self::Output<T>;
}
impl Prepend for Nil {
  type Output<T> = Cons<T, Self>;
  #[inline]
  fn prepend<T>(self, value: T) -> Self::Output<T> { Cons::new(value, self) }
}
impl<Head, Rest: List> Prepend for Cons<Head, Rest> {
  type Output<T> = Cons<T, Self>;
  #[inline]
  fn prepend<T>(self, value: T) -> Self::Output<T> { Cons::new(value, self) }
}

/// Fold over list
pub trait Fold<A, F>: List {
  fn fold(self, accumulator: A, folder: F) -> A;
}
// Generic fold impl
impl<A, F> Fold<A, F> for Nil {
  #[inline]
  fn fold(self, init: A, _folder: F) -> A { init }
}
// Homogenous fold impl
impl<A, F, Head, Tail> Fold<A, F> for Cons<Head, Tail> where
  F: FnMut(A, Head) -> A,
  Tail: Fold<A, F>,
{
  #[inline]
  fn fold(self, accumulator: A, mut folder: F) -> A {
    let accumulator = folder(accumulator, self.head);
    self.tail.fold(accumulator, folder)
  }
}
// Heterogeneous fold impl
pub struct Folder<T>(pub T); // For preventing conflicting implementation.
pub trait FolderFn<A, T> {
  fn fold(&mut self, accumulator: A, value: T) -> A;
}
impl<A, F, Head, Tail> Fold<A, Folder<F>> for Cons<Head, Tail> where
  F: FolderFn<A, Head>,
  Tail: Fold<A, Folder<F>>,
{
  #[inline]
  fn fold(self, accumulator: A, mut folder: Folder<F>) -> A {
    let accumulator = folder.0.fold(accumulator, self.head);
    self.tail.fold(accumulator, folder)
  }
}

/// Convert list to list of (mutable) references
pub trait ToRef: List {
  type Ref<'a>: List where Self: 'a;
  fn to_ref(&self) -> Self::Ref<'_>;

  // type Mut<'a>: List where Self: 'a;
  // fn to_mut(&mut self) -> Self::Mut<'_>;
}
impl ToRef for Nil {
  type Ref<'a> = Nil;
  #[inline]
  fn to_ref(&self) -> Self::Ref<'_> { *self }

  // type Mut<'a> = Nil where Self: 'a;
  // #[inline]
  // fn to_mut(&mut self) -> Self::Mut<'_> { *self }
}
impl<Head, Tail: ToRef> ToRef for Cons<Head, Tail> {
  type Ref<'a> = Cons<&'a Head, Tail::Ref<'a>> where Self: 'a;
  #[inline]
  fn to_ref(&self) -> Self::Ref<'_> {
    Cons::new(&self.head, self.tail.to_ref())
  }

  // type Mut<'a> = Cons<&'a mut Head, Tail::Mut<'a>> where Self: 'a;
  // #[inline]
  // fn to_mut(&mut self) -> Self::Mut<'_> {
  //   Cons::new(&mut self.head, self.tail.to_mut())
  // }
}
