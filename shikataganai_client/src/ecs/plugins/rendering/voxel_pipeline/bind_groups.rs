use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use bevy::render::render_resource::BindGroup;

#[derive(Resource, Clone, ExtractResource)]
pub struct TextureHandle(pub Handle<Image>);

#[derive(Resource, Clone, ExtractResource)]
pub struct ArrayTextureHandle(pub Handle<Image>);

#[derive(Resource, Clone, ExtractResource)]
pub struct ItemTextureHandle(pub Handle<Image>);

#[derive(Clone, ExtractResource, Resource)]
pub struct LightTextureHandle(pub Handle<Image>);

impl FromWorld for TextureHandle {
  fn from_world(world: &mut World) -> Self {
    let asset_server = world.resource_mut::<AssetServer>();
    TextureHandle(asset_server.load("texture.png"))
  }
}

impl FromWorld for ItemTextureHandle {
  fn from_world(world: &mut World) -> Self {
    let asset_server = world.resource_mut::<AssetServer>();
    ItemTextureHandle(asset_server.load("item.png"))
  }
}

impl FromWorld for LightTextureHandle {
  fn from_world(world: &mut World) -> Self {
    let asset_server = world.resource_mut::<AssetServer>();
    LightTextureHandle(asset_server.load("light.png"))
  }
}

#[derive(Deref, Resource)]
pub struct VoxelViewBindGroup {
  pub bind_group: BindGroup,
}

#[derive(Deref, Resource)]
pub struct VoxelTextureBindGroup {
  pub bind_group: BindGroup,
}

#[derive(Deref, Resource)]
pub struct SelectionBindGroup {
  pub bind_group: BindGroup,
}

#[derive(Deref, Resource)]
pub struct LightTextureBindGroup {
  pub bind_group: BindGroup,
}
