use bevy::prelude::Vec3;
use serde::ser::SerializeTuple;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::alloc::Layout;
use std::array::IntoIter;
use std::fmt::Debug;
use std::ops::{Index, IndexMut};

pub type DD = (i32, i32);
pub type DDD = (i32, i32, i32);

#[inline]
pub fn neg_ddd(a: DDD) -> DDD {
  (-a.0, -a.1, -a.2)
}

#[inline]
pub fn add_ddd(a: DDD, b: DDD) -> DDD {
  (a.0 + b.0, a.1 + b.1, a.2 + b.2)
}

#[inline]
pub fn sub_ddd(a: DDD, b: DDD) -> DDD {
  (a.0 - b.0, a.1 - b.1, a.2 - b.2)
}

pub trait ImmediateNeighbours {
  fn immediate_neighbours(&self) -> IntoIter<Self, 6>
  where
    Self: Sized;
}

pub trait FullNeighbours {
  type It: Iterator<Item = Self>;
  fn full_neighbours(&self) -> Self::It
  where
    Self: Sized;
}

impl ImmediateNeighbours for DDD {
  fn immediate_neighbours(&self) -> IntoIter<Self, 6> {
    [
      (self.0 - 1, self.1, self.2),
      (self.0 + 1, self.1, self.2),
      (self.0, self.1 - 1, self.2),
      (self.0, self.1 + 1, self.2),
      (self.0, self.1, self.2 - 1),
      (self.0, self.1, self.2 + 1),
    ]
    .into_iter()
  }
}

pub struct FullNeighoursIter<T> {
  base: T,
  offset: Option<T>,
}

impl Iterator for FullNeighoursIter<DDD> {
  type Item = DDD;

  fn next(&mut self) -> Option<Self::Item> {
    match self.offset {
      None => None,
      Some(offset) => {
        let next = add_ddd(self.base, offset);
        self.offset = offset.next(&((-1, -1, -1), (1, 1, 1)));
        Some(next)
      }
    }
  }
}

impl FullNeighbours for DDD {
  type It = FullNeighoursIter<DDD>;

  fn full_neighbours(&self) -> Self::It
  where
    Self: Sized,
  {
    FullNeighoursIter {
      base: *self,
      offset: Some((-1, -1, -1)),
    }
  }
}

pub fn to_ddd(v: Vec3) -> DDD {
  (v.x.floor() as i32, v.y.floor() as i32, v.z.floor() as i32)
}

pub fn from_ddd(v: DDD) -> Vec3 {
  Vec3::new(v.0 as f32, v.1 as f32, v.2 as f32)
}

pub type Bounds<T> = (T, T);

pub trait ArrayIndex
where
  Self: Sized,
{
  fn size(bounds: &Bounds<Self>) -> usize;
  fn idx(&self, bounds: &Bounds<Self>) -> usize;
  fn next(self, bounds: &Bounds<Self>) -> Option<Self>;
  fn in_bounds(&self, other: &Bounds<Self>) -> bool;
}

impl ArrayIndex for DD {
  fn size(bounds: &Bounds<Self>) -> usize {
    let ((x1, y1), (x2, y2)) = bounds;
    ((x2 - x1 + 1) * (y2 - y1 + 1)) as usize
  }

  fn idx(&self, bounds: &Bounds<Self>) -> usize {
    #[cfg(debug_assertions)]
    assert!(self.in_bounds(bounds), "Array index out of bounds");
    let ((x1, y1), (x2, _)) = bounds;
    let (x, y) = self;
    let row = x2 - x1 + 1;
    ((y - y1) * row + (x - x1)) as usize
  }

  fn next(self, bounds: &Bounds<Self>) -> Option<Self> {
    let ((x1, _), (x2, y2)) = bounds;
    let (mut x, mut y) = self;
    x += 1;
    if x > *x2 {
      x = *x1;
      y += 1;
    }
    if y > *y2 {
      None
    } else {
      Some((x, y))
    }
  }

  fn in_bounds(&self, other: &Bounds<Self>) -> bool {
    let ((x1, y1), (x2, y2)) = other;
    let (x, y) = self;
    x >= x1 && x <= x2 && y >= y1 && y <= y2
  }
}

pub struct Array<I: ArrayIndex + Copy + Debug + Serialize, T: Serialize> {
  data: *mut T,
  pub bounds: (I, I),
}

impl<I: ArrayIndex + Copy + Debug + Serialize, T: Serialize> Serialize for Array<I, T> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut tuple = serializer.serialize_tuple(2)?;
    tuple.serialize_element(&self.bounds)?;
    unsafe {
      tuple.serialize_element(&std::slice::from_raw_parts(self.data, ArrayIndex::size(&self.bounds)))?;
    }
    tuple.end()
  }
}

impl<'de, I: ArrayIndex + Copy + Debug + Serialize + Deserialize<'de>, T: Serialize + Deserialize<'de>> Deserialize<'de>
  for Array<I, T>
{
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let (bounds, data): ((I, I), Vec<T>) = Deserialize::deserialize(deserializer)?;
    let capacity = ArrayIndex::size(&bounds);
    assert_eq!(capacity, data.len(), "Vector bounds mismatch");
    let array = Array::new_zeroed(bounds);
    unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), array.data, capacity) };
    Ok(array)
  }
}

impl ArrayIndex for DDD {
  fn size(bounds: &Bounds<Self>) -> usize {
    let ((x1, y1, z1), (x2, y2, z2)) = bounds;
    ((x2 - x1 + 1) * (y2 - y1 + 1) * (z2 - z1 + 1)) as usize
  }

  fn idx(&self, bounds: &Bounds<Self>) -> usize {
    #[cfg(debug_assertions)]
    assert!(self.in_bounds(bounds), "Array index out of bounds {self:?} {bounds:?}");
    let ((x1, y1, z1), (x2, y2, _)) = bounds;
    let (x, y, z) = self;
    let row = x2 - x1 + 1;
    let slice = y2 - y1 + 1;
    ((z - z1) * row * slice + (y - y1) * row + (x - x1)) as usize
  }

  fn next(self, bounds: &Bounds<Self>) -> Option<Self> {
    let ((x1, y1, _), (x2, y2, z2)) = bounds;
    let (mut x, mut y, mut z) = self;
    x += 1;
    if x > *x2 {
      x = *x1;
      y += 1;
    }
    if y > *y2 {
      y = *y1;
      z += 1;
    }
    if z > *z2 {
      None
    } else {
      Some((x, y, z))
    }
  }

  fn in_bounds(&self, other: &Bounds<Self>) -> bool {
    let ((x1, y1, z1), (x2, y2, z2)) = other;
    let (x, y, z) = self;
    x >= x1 && x <= x2 && y >= y1 && y <= y2 && z >= z1 && z <= z2
  }
}

pub type Array2d<T> = Array<DD, T>;
pub type Array3d<T> = Array<DDD, T>;

impl<I: Copy + ArrayIndex + Debug + Serialize, T: Copy + Serialize> Clone for Array<I, T> {
  fn clone(&self) -> Self {
    let ptr = unsafe {
      let size = ArrayIndex::size(&self.bounds);
      let ptr = std::alloc::alloc(Layout::array::<T>(size).unwrap()) as *mut T;
      std::ptr::copy_nonoverlapping(self.data, ptr, size);
      ptr
    };
    Self {
      data: ptr,
      bounds: self.bounds,
    }
  }
}

impl<I: Copy + ArrayIndex + Debug + Serialize, T: Serialize> Drop for Array<I, T> {
  fn drop(&mut self) {
    let size = ArrayIndex::size(&self.bounds);
    unsafe { std::alloc::dealloc(self.data as *mut u8, Layout::array::<T>(size).unwrap()) };
  }
}

impl<I: Copy + ArrayIndex + Debug + Serialize, T: Default + Serialize> Array<I, T> {
  pub fn zero_out(&mut self) {
    self.map_in_place(|_, _| T::default())
  }
}

impl<I: Copy + ArrayIndex + Debug + Serialize, T: Serialize> Array<I, T> {
  pub fn new_zeroed((from, to): (I, I)) -> Array<I, T> {
    let size = ArrayIndex::size(&(from, to));
    let ptr = unsafe { std::alloc::alloc_zeroed(Layout::array::<T>(size).unwrap()) } as *mut T;
    Self {
      data: ptr,
      bounds: (from, to),
    }
  }
  pub fn new_init<F: FnMut(I) -> T>(bounds: (I, I), mut f: F) -> Array<I, T> {
    let mut array = Self::new_zeroed(bounds);
    let mut i = bounds.0;
    loop {
      array[i] = f(i);
      i = match i.next(&bounds) {
        None => break,
        Some(i) => i,
      };
    }
    array
  }
  pub unsafe fn data(&self) -> *const T {
    self.data as *const T
  }
  pub fn size(&self) -> usize {
    ArrayIndex::size(&self.bounds)
  }
  pub fn in_bounds(&self, test: I) -> bool {
    test.in_bounds(&self.bounds)
  }
  pub fn map_in_place<F: Fn(I, &T) -> T>(&mut self, f: F) {
    let mut i = self.bounds.0;
    loop {
      self[i] = f(i, &self[i]);
      i = match i.next(&self.bounds) {
        None => break,
        Some(i) => i,
      };
    }
  }
  pub fn map<O: Serialize, F: Fn(I, &T) -> O>(&self, f: F) -> Array<I, O> {
    Array::new_init(self.bounds, |x| f(x, &self[x]))
  }
  pub fn foreach<F: FnMut(I, &T)>(&self, mut f: F) {
    let mut i = self.bounds.0;
    loop {
      f(i, &self[i]);
      i = match i.next(&self.bounds) {
        None => break,
        Some(i) => i,
      };
    }
  }
  pub fn as_slice(&self) -> &[u8] {
    let size = ArrayIndex::size(&self.bounds);
    unsafe { std::slice::from_raw_parts(self.data as *const u8, size as usize) }
  }
}

impl<I: Copy + ArrayIndex + Debug + Serialize, T: Serialize> Index<I> for Array<I, T> {
  type Output = T;

  fn index(&self, i: I) -> &Self::Output {
    unsafe { &*self.data.add(i.idx(&self.bounds)) }
  }
}

impl<I: Copy + ArrayIndex + Debug + Serialize, T: Serialize> IndexMut<I> for Array<I, T> {
  fn index_mut(&mut self, i: I) -> &mut Self::Output {
    unsafe { &mut *self.data.add(i.idx(&self.bounds)) }
  }
}

unsafe impl<I: Copy + ArrayIndex + Debug + Serialize, T: Serialize> Send for Array<I, T> {}
unsafe impl<I: Copy + ArrayIndex + Debug + Serialize, T: Serialize> Sync for Array<I, T> {}
