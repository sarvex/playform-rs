//! Position data structure for terrain blocks.

use cgmath::{Point3, Vector3};
use std::ops::Add;
use voxel_data;

use terrain_block;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
/// Position of blocks on an "infinite" regular grid.
/// The position is implicitly in units of terrain_block::WIDTH.
pub struct T(Point3<i32>);

pub fn containing_voxel(bounds: &voxel_data::bounds::T) -> T {
  if bounds.lg_size < 0 {
    new(
      bounds.x >> -bounds.lg_size >> terrain_block::LG_WIDTH,
      bounds.y >> -bounds.lg_size >> terrain_block::LG_WIDTH,
      bounds.z >> -bounds.lg_size >> terrain_block::LG_WIDTH,
    )
  } else {
    new(
      (bounds.x << bounds.lg_size) >> terrain_block::LG_WIDTH,
      (bounds.y << bounds.lg_size) >> terrain_block::LG_WIDTH,
      (bounds.z << bounds.lg_size) >> terrain_block::LG_WIDTH,
    )
  }
}

impl T {
  #[allow(missing_docs)]
  pub fn as_pnt(&self) -> &Point3<i32> {
    let T(ref pnt) = *self;
    pnt
  }

  #[allow(missing_docs)]
  pub fn as_mut_pnt(&mut self) -> &mut Point3<i32> {
    let T(ref mut pnt) = *self;
    pnt
  }

  #[allow(missing_docs)]
  #[allow(dead_code)]
  pub fn to_world_position(&self) -> Point3<f32> {
    Point3::new(
      (self.as_pnt().x * terrain_block::WIDTH) as f32,
      (self.as_pnt().y * terrain_block::WIDTH) as f32,
      (self.as_pnt().z * terrain_block::WIDTH) as f32,
    )
  }
}

#[allow(missing_docs)]
pub fn new(x: i32, y: i32, z: i32) -> T {
  T(Point3::new(x, y, z))
}

#[allow(missing_docs)]
pub fn of_pnt(p: &Point3<i32>) -> T {
  T(p.clone())
}

#[allow(missing_docs)]
pub fn of_world_position(world_position: &Point3<f32>) -> T {
  macro_rules! convert_coordinate(
    ($x: expr) => ({
      let x = $x.floor() as i32;
      let x =
        if x < 0 {
          x - (terrain_block::WIDTH - 1)
        } else {
          x
        };
      x / terrain_block::WIDTH
    })
  );
  T(
    Point3::new(
      convert_coordinate!(world_position.x),
      convert_coordinate!(world_position.y),
      convert_coordinate!(world_position.z),
    )
  )
}

impl Add<Vector3<i32>> for T {
  type Output = T;

  fn add(mut self, rhs: Vector3<i32>) -> Self {
    self.as_mut_pnt().x += rhs.x;
    self.as_mut_pnt().y += rhs.y;
    self.as_mut_pnt().z += rhs.z;
    self
  }
}