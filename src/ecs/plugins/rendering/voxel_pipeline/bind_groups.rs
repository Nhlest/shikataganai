use bevy::prelude::*;
use bevy::render::render_resource::BindGroup;

pub struct TextureHandle(pub Handle<Image>);
pub struct LightTextureHandle(pub Handle<Image>);

impl FromWorld for TextureHandle {
  fn from_world(world: &mut World) -> Self {
    let asset_server = world.resource_mut::<AssetServer>();
    TextureHandle(asset_server.load("texture.png"))
  }
}

impl FromWorld for LightTextureHandle {
  fn from_world(world: &mut World) -> Self {
    let asset_server = world.resource_mut::<AssetServer>();
    LightTextureHandle(asset_server.load("light.png"))
  }
}

#[derive(Deref)]
pub struct VoxelViewBindGroup {
  pub bind_group: BindGroup,
}

#[derive(Deref)]
pub struct VoxelTextureBindGroup {
  pub bind_group: BindGroup,
}

#[derive(Deref)]
pub struct SelectionBindGroup {
  pub bind_group: BindGroup,
}

#[derive(Deref)]
pub struct LightTextureBindGroup {
  pub bind_group: BindGroup,
}
