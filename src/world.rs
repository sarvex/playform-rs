use id_allocator::IdAllocator;
use mob;
use ncollide_entities::bounding_volume::AABB3;
use physics::Physics;
use player::Player;
use std::cell::RefCell;
use std::collections::HashMap;
use std::default::Default;
use std::ops::Add;
use std::rc::Rc;
use sun::Sun;
use terrain::terrain_game_loader::TerrainGameLoader;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct EntityId(u32);

impl Default for EntityId {
  fn default() -> EntityId {
    EntityId(0)
  }
}

impl Add<u32> for EntityId {
  type Output = EntityId;

  fn add(self, rhs: u32) -> EntityId {
    let EntityId(i) = self;
    EntityId(i + rhs)
  }
}

pub struct World<'a> {
  pub physics: Physics,
  pub player: Player<'a>,
  pub mobs: HashMap<EntityId, Rc<RefCell<mob::Mob<'a>>>>,
  pub sun: Sun,

  pub id_allocator: IdAllocator<EntityId>,
  pub terrain_game_loader: TerrainGameLoader,
}

impl<'a> World<'a> {
  #[inline]
  pub fn get_bounds(&self, id: EntityId) -> &AABB3<f32> {
    self.physics.get_bounds(id).unwrap()
  }
}