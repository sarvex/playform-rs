//! This crate contains the terrain data structures and generation.

#![allow(let_and_return)]
#![allow(match_ref_pats)]
#![allow(type_complexity)]

#![deny(missing_docs)]
#![deny(warnings)]

#![feature(main)]
#![feature(plugin)]
#![feature(test)]
#![feature(unboxed_closures)]

#![plugin(clippy)]

extern crate cgmath;
extern crate common;
#[macro_use]
extern crate log;
extern crate noise;
extern crate rand;
extern crate stopwatch;
extern crate test;
extern crate time;
extern crate voxel_data;
extern crate num;

pub mod biome;
pub mod tree;

pub use noise::Seed;

use cgmath::Aabb;
use std::sync::Mutex;

use common::voxel;

/// This struct contains and lazily generates the world's terrain.
#[allow(missing_docs)]
pub struct T {
  pub mosaic: Box<voxel_data::mosaic::T<voxel::Material> + Sync>,
  pub voxels: Mutex<voxel::tree::T>,
}

impl T {
  #[allow(missing_docs)]
  pub fn new(terrain_seed: Seed) -> T {
    T {
      mosaic: Box::new(biome::hills::new(terrain_seed)),
      voxels: Mutex::new(voxel::tree::T::new()),
    }
  }

  /// Load the block of terrain at a given position.
  // TODO: Allow this to be performed in such a way that self is only briefly locked.
  pub fn load<F>(
    &self,
    bounds: &voxel::bounds::T,
    mut f: F
  ) where
    F: FnMut(&voxel::T)
  {
    let mut voxels = self.voxels.lock().unwrap();
    let branch = voxels.get_mut_or_create(bounds);
    match branch {
      &mut voxel_data::tree::Empty => {
        let voxel = voxel::unwrap(voxel::of_field(&self.mosaic, bounds));
        f(&voxel);
        *branch =
          voxel_data::tree::Branch {
            data: Some(voxel.clone()),
            branches: Box::new(voxel_data::tree::Branches::empty()),
          };
      },
      &mut voxel_data::tree::Branch { ref mut data, branches: _ }  => {
        match data {
          &mut None => {
            let voxel = voxel::unwrap(voxel::of_field(&self.mosaic, bounds));
            f(&voxel);
            *data = Some(voxel);
          },
          &mut Some(ref data) => {
            f(data);
          },
        }
      },
    }
  }

  /// Apply a voxel brush to the terrain.
  pub fn brush<VoxelChanged, Mosaic>(
    &self,
    brush: &voxel_data::brush::T<Mosaic>,
    mut voxel_changed: VoxelChanged,
  ) where
    VoxelChanged: FnMut(&voxel::T, &voxel::bounds::T),
    Mosaic: voxel_data::mosaic::T<voxel::Material>,
  {
    let mut voxels = self.voxels.lock().unwrap();
    voxels.brush(
      brush,
      &mut |bounds| Some(voxel::unwrap(voxel::of_field(&self.mosaic, bounds))),
      &mut voxel_changed,
    );
  }
}
