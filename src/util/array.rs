use std::alloc::Layout;
use std::fmt::{Debug, Formatter};
use std::ops::{Index, IndexMut};

pub struct Array2d<T> {
  data: *mut T,
  pub bounds: ((i32, i32), (i32, i32)),
}

impl<T: Copy> Clone for Array2d<T> {
  fn clone(&self) -> Self {
    let ((x0, y0), (x1, y1)) = self.bounds;
    let ptr = unsafe {
      let size = ((x1 - x0 + 1) * (y1 - y0 + 1)) as usize;
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

impl<T: Debug> Debug for Array2d<T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let ((x0, y0), (x1, y1)) = self.bounds;
    for iy in y0..=y1 {
      for ix in x0..=x1 {
        f.write_str(format!("{:3?}", self[(ix, iy)]).as_str()).unwrap();
      }
      f.write_str("\n").unwrap();
    }
    Ok(())
  }
}

impl<T> Drop for Array2d<T> {
  fn drop(&mut self) {
    let ((x0, y0), (x1, y1)) = self.bounds;
    let size = ((x1 - x0 + 1) * (y1 - y0 + 1)) as usize;
    unsafe { std::alloc::dealloc(self.data as *mut u8, Layout::array::<T>(size).unwrap()) };
  }
}

impl<T> Array2d<T> {
  pub fn new_zeroed(((x0, y0), (x1, y1)): ((i32, i32), (i32, i32))) -> Array2d<T> {
    let size = ((x1 - x0 + 1) * (y1 - y0 + 1)) as usize;
    let ptr = unsafe { std::alloc::alloc_zeroed(Layout::array::<T>(size).unwrap()) } as *mut T;
    Self {
      data: ptr,
      bounds: ((x0, y0), (x1, y1)),
    }
  }
  pub fn new_init<F: FnMut((i32, i32)) -> T>(((x0, y0), (x1, y1)): ((i32, i32), (i32, i32)), mut f: F) -> Array2d<T> {
    let mut array = Self::new_zeroed(((x0, y0), (x1, y1)));
    for iy in y0..=y1 {
      for ix in x0..=x1 {
        array[(ix, iy)] = f((ix, iy));
      }
    }
    array
  }
  pub fn in_bounds(&self, (x, y): (i32, i32)) -> bool {
    let ((x0, y0), (x1, y1)) = self.bounds;
    x >= x0 && x <= x1 && y >= y0 && y <= y1
  }
  pub fn map_in_place<F: Fn((i32, i32), &T) -> T>(&mut self, f: F) {
    let ((x1, y1), (x2, y2)) = self.bounds;
    for ix in x1..=x2 {
      for iy in y1..=y2 {
        self[(ix, iy)] = f((ix, iy), &self[(ix, iy)]);
      }
    }
  }
  pub fn map<O, F: Fn((i32, i32), &T) -> O>(&self, f: F) -> Array2d<O> {
    Array2d::new_init(self.bounds, |x| f(x, &self[x]))
  }
  pub fn foreach<F: FnMut((i32, i32), &T)>(&self, mut f: F) {
    let ((x1, y1), (x2, y2)) = self.bounds;
    for ix in x1..=x2 {
      for iy in y1..=y2 {
        f((ix, iy), &self[(ix, iy)]);
      }
    }
  }
}

impl<T> Index<(i32, i32)> for Array2d<T> {
  type Output = T;

  fn index(&self, (x, y): (i32, i32)) -> &Self::Output {
    let ((x0, y0), (x1, y1)) = self.bounds;
    assert!(x0 <= x && x <= x1 && y0 <= y && y <= y1, "Array out of bounds");
    let row = x1 - x0 + 1;
    unsafe { &*self.data.add(((y - y0) * row + (x - x0)) as usize) }
  }
}

impl<T> IndexMut<(i32, i32)> for Array2d<T> {
  fn index_mut(&mut self, (x, y): (i32, i32)) -> &mut Self::Output {
    let ((x0, y0), (x1, y1)) = self.bounds;
    assert!(x0 <= x && x <= x1 && y0 <= y && y <= y1, "Array out of bounds");
    let row = x1 - x0 + 1;
    unsafe { &mut *self.data.add(((y - y0) * row + (x - x0)) as usize) }
  }
}

unsafe impl<T> Send for Array2d<T> {}
unsafe impl<T> Sync for Array2d<T> {}

pub struct Array<T> {
  data: *mut T,
  pub bounds: ((i32, i32, i32), (i32, i32, i32)),
}

impl<T: Copy> Clone for Array<T> {
  fn clone(&self) -> Self {
    let ((x0, y0, z0), (x1, y1, z1)) = self.bounds;
    let ptr = unsafe {
      let size = ((x1 - x0 + 1) * (y1 - y0 + 1) * (z1 - z0 + 1)) as usize;
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

impl<T> Drop for Array<T> {
  fn drop(&mut self) {
    let ((x0, y0, z0), (x1, y1, z1)) = self.bounds;
    let size = ((x1 - x0 + 1) * (y1 - y0 + 1) * (z1 - z0 + 1)) as usize;
    unsafe { std::alloc::dealloc(self.data as *mut u8, Layout::array::<T>(size).unwrap()) };
  }
}

impl<T: Default> Array<T> {
  pub fn zero_out(&mut self) {
    self.map_in_place(|_, _| T::default())
  }
}

impl<T> Array<T> {
  pub unsafe fn data(&self) -> *const T {
    self.data
  }
  pub fn size(&self) -> usize {
    let ((x1, y1, z1), (x2, y2, z2)) = self.bounds;
    ((x2 - x1 + 1) * (y2 - y1 + 1) * (z2 - z1 + 1)) as usize
  }
  pub fn new_zeroed(((x0, y0, z0), (x1, y1, z1)): ((i32, i32, i32), (i32, i32, i32))) -> Array<T> {
    let size = ((x1 - x0 + 1) * (y1 - y0 + 1) * (z1 - z0 + 1)) as usize;
    let ptr = unsafe { std::alloc::alloc_zeroed(Layout::array::<T>(size).unwrap()) } as *mut T;
    Self {
      data: ptr,
      bounds: ((x0, y0, z0), (x1, y1, z1)),
    }
  }
  pub fn new_init<F: Fn((i32, i32, i32)) -> T>(
    ((x0, y0, z0), (x1, y1, z1)): ((i32, i32, i32), (i32, i32, i32)),
    f: F,
  ) -> Array<T> {
    let mut array = Self::new_zeroed(((x0, y0, z0), (x1, y1, z1)));
    for iz in z0..=z1 {
      for iy in y0..=y1 {
        for ix in x0..=x1 {
          array[(ix, iy, iz)] = f((ix, iy, iz));
        }
      }
    }
    array
  }
  pub fn as_slice(&self) -> &[u8] {
    let ((x1, y1, z1), (x2, y2, z2)) = self.bounds;
    let size = (x2 - x1 + 1) * (y2 - y1 + 1) * (z2 - z1 + 1) * std::mem::size_of::<T>() as i32;
    unsafe { std::slice::from_raw_parts(self.data as *const u8, size as usize) }
  }
  pub fn in_bounds(&self, (x, y, z): (i32, i32, i32)) -> bool {
    let ((x0, y0, z0), (x1, y1, z1)) = self.bounds;
    x >= x0 && x <= x1 && y >= y0 && y <= y1 && z >= z0 && z <= z1
  }
  pub fn map_in_place<F: Fn((i32, i32, i32), &T) -> T>(&mut self, f: F) {
    let ((x1, y1, z1), (x2, y2, z2)) = self.bounds;
    for ix in x1..=x2 {
      for iy in y1..=y2 {
        for iz in z1..=z2 {
          self[(ix, iy, iz)] = f((ix, iy, iz), &self[(ix, iy, iz)]);
        }
      }
    }
  }
  pub fn map<O, F: Fn((i32, i32, i32), &T) -> O>(&self, f: F) -> Array<O> {
    Array::new_init(self.bounds, |x| f(x, &self[x]))
  }
  pub fn foreach<F: FnMut((i32, i32, i32), &T)>(&self, mut f: F) {
    let ((x1, y1, z1), (x2, y2, z2)) = self.bounds;
    for ix in x1..=x2 {
      for iy in y1..=y2 {
        for iz in z1..=z2 {
          f((ix, iy, iz), &self[(ix, iy, iz)]);
        }
      }
    }
  }
}

impl<T> Index<(i32, i32, i32)> for Array<T> {
  type Output = T;

  fn index(&self, (x, y, z): (i32, i32, i32)) -> &Self::Output {
    let ((x0, y0, z0), (x1, y1, z1)) = self.bounds;
    assert!(
      x0 <= x && x <= x1 && y0 <= y && y <= y1 && z0 <= z && z <= z1,
      "Array out of bounds"
    );
    let row = x1 - x0 + 1;
    let slice = (y1 - y0 + 1) * row;
    unsafe { &*self.data.add(((z - z0) * slice + (y - y0) * row + (x - x0)) as usize) }
  }
}

impl<T> IndexMut<(i32, i32, i32)> for Array<T> {
  fn index_mut(&mut self, (x, y, z): (i32, i32, i32)) -> &mut Self::Output {
    let ((x0, y0, z0), (x1, y1, z1)) = self.bounds;
    assert!(
      x0 <= x && x <= x1 && y0 <= y && y <= y1 && z0 <= z && z <= z1,
      "Array out of bounds"
    );
    let row = x1 - x0 + 1;
    let slice = (y1 - y0 + 1) * row;
    unsafe { &mut *self.data.add(((z - z0) * slice + (y - y0) * row + (x - x0)) as usize) }
  }
}

unsafe impl<T> Send for Array<T> {}
unsafe impl<T> Sync for Array<T> {}
