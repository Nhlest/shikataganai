use bevy::ecs::component::Component;
use bevy::render::render_resource::encase::internal::{BufferMut, WriteInto, Writer};
use bevy::render::render_resource::encase::private::Metadata;
use bevy::render::render_resource::ShaderType;

impl ShaderType for LightLevel {
  type ExtraMetadata = ();
  const METADATA: Metadata<()> = Metadata::from_alignment_and_size(8, 8);
}

impl WriteInto for LightLevel {
  fn write_into<B>(&self, writer: &mut Writer<B>)
  where
    B: BufferMut,
  {
    writer.write(&[self.hearth, 0, 0, 0, self.heaven, 0, 0, 0])
  }
}

#[derive(Copy, Clone, Component, Debug)]
pub struct LightLevel {
  pub heaven: u8,
  pub hearth: u8,
}

impl LightLevel {
  pub fn new(heaven: u8, hearth: u8) -> Self {
    Self { heaven, hearth }
  }
  pub fn dark() -> Self {
    Self { heaven: 0, hearth: 0 }
  }
}
