//! Chunk type

use cgmath;
use std;

use voxel;

/// Width of a chunk, in voxels
pub const WIDTH: u32 = 1 << LG_WIDTH;
/// Base-2 log of the chunk width
pub const LG_WIDTH: u16 = 5;

/// A position in "chunk coordinates".
pub mod position {
  use cgmath;

  #[derive(Debug, Clone, RustcEncodable, RustcDecodable, PartialEq, Eq, Hash)]
  #[allow(missing_docs)]
  pub struct T {
    pub coords        : cgmath::Point3<i32>,
    pub lg_voxel_size : i16,
  }

  impl T {
    /// Return an iterator for the bounds of the voxels in a chunk.
    pub fn voxels(&self) -> super::VoxelBounds {
      super::VoxelBounds::new(self)
    }
  }
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
#[allow(missing_docs)]
pub struct T {
  pub position : position::T,
  pub voxels   : Vec<voxel::T>,
}

impl T {
  fn idx(&self, p: &cgmath::Point3<i32>) -> usize {
    (p.x as usize * WIDTH as usize + p.y as usize) * WIDTH as usize + p.z as usize
  }

  #[allow(missing_docs)]
  pub fn get<'a>(&'a self, p: &cgmath::Point3<i32>) -> &'a voxel::T {
    let idx = self.idx(p);
    &self.voxels[idx]
  }

  #[allow(missing_docs)]
  pub fn get_mut<'a>(&'a mut self, p: &cgmath::Point3<i32>) -> &'a voxel::T {
    let idx = self.idx(p);
    &mut self.voxels[idx]
  }

  /// Iterate through the voxels in this chunk.
  pub fn voxels(&self) -> Voxels {
    Voxels::new(self)
  }
}

/// Construct a chunk from a position and an initialization callback.
pub fn of_callback<F>(p: &position::T, mut f: F) -> T
  where F: FnMut(voxel::bounds::T) -> voxel::T
{
  assert!(p.lg_voxel_size <= 0 || p.lg_voxel_size as u16 <= LG_WIDTH);

  let mut voxels = Vec::new();

  let samples = 1 << (LG_WIDTH as i16 - p.lg_voxel_size);
  for x in 0 .. samples {
  for y in 0 .. samples {
  for z in 0 .. samples {
    let bounds =
      voxel::bounds::T {
        x: p.coords.x + x,
        y: p.coords.y + y,
        z: p.coords.z + z,
        lg_size: p.lg_voxel_size,
      };
    voxels.push(f(bounds));
  }}}

  T {
    position: p.clone(),
    voxels: voxels,
  }
}

/// An iterator for the bounds of the voxels inside a chunk.
pub struct VoxelBounds<'a> {
  chunk   : &'a position::T,
  current : cgmath::Point3<u8>,
  done    : bool,
}

impl<'a> VoxelBounds <'a> {
  #[allow(missing_docs)]
  pub fn new<'b:'a>(chunk: &'b position::T) -> Self {
    VoxelBounds {
      chunk         : chunk,
      current       : cgmath::Point3::new(0, 0, 0),
      done          : false,
    }
  }
}

impl<'a> std::iter::Iterator for VoxelBounds<'a> {
  type Item = voxel::bounds::T;
  fn next(&mut self) -> Option<Self::Item> {
    if self.done {
      return None
    }

    let r =
      Some(
        voxel::bounds::T {
          x       : WIDTH as i32 * self.chunk.coords.x + self.current.x as i32,
          y       : WIDTH as i32 * self.chunk.coords.y + self.current.y as i32,
          z       : WIDTH as i32 * self.chunk.coords.z + self.current.z as i32,
          lg_size : self.chunk.lg_voxel_size,
        },
      );

    self.current.x += 1;
    if (self.current.x as u32) < WIDTH { return r }
    self.current.x = 0;

    self.current.y += 1;
    if (self.current.y as u32) < WIDTH { return r }
    self.current.y = 0;

    self.current.z += 1;
    if (self.current.z as u32) < WIDTH { return r }
    self.done = true;

    r
  }
}

/// An iterator for the voxels inside a chunk.
pub struct Voxels<'a> {
  chunk   : &'a T,
  current : cgmath::Point3<u8>,
  done    : bool,
}

impl<'a> Voxels <'a> {
  #[allow(missing_docs)]
  pub fn new<'b: 'a>(chunk: &'b T) -> Self {
    Voxels {
      chunk   : chunk,
      current : cgmath::Point3::new(0, 0, 0),
      done    : false,
    }
  }
}

impl<'a> std::iter::Iterator for Voxels<'a> {
  type Item = (voxel::bounds::T, voxel::T);
  fn next(&mut self) -> Option<Self::Item> {
    if self.done {
      return None
    }

    let x = self.chunk.position.coords.x + self.current.x as i32;
    let y = self.chunk.position.coords.y + self.current.y as i32;
    let z = self.chunk.position.coords.z + self.current.z as i32;
    let r =
      Some((
        voxel::bounds::T { x: x, y: y, z: z, lg_size: self.chunk.position.lg_voxel_size },
        *self.chunk.get(&cgmath::Point3::new(x, y, z)),
      ));

    self.current.z += 1;
    if (self.current.z as u32) < WIDTH { return r }
    self.current.z = 0;

    self.current.y += 1;
    if (self.current.y as u32) < WIDTH { return r }
    self.current.y = 0;

    self.current.x += 1;
    if (self.current.x as u32) < WIDTH { return r }
    self.done = true;

    r
  }
}
