use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, MeshVertexAttribute, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_resource::VertexFormat;
use bevy::utils::hashbrown::HashMap;
use bevy::utils::BoxedFuture;
use gltf::buffer::Source;
use gltf::Gltf;
use std::path::Path;
use strum_macros::EnumIter;

#[derive(Default)]
pub struct GltfLoaderII;

struct DataUri<'a> {
  mime_type: &'a str,
  base64: bool,
  data: &'a str,
}

fn split_once(input: &str, delimiter: char) -> Option<(&str, &str)> {
  let mut iter = input.splitn(2, delimiter);
  Some((iter.next()?, iter.next()?))
}

impl<'a> DataUri<'a> {
  fn parse(uri: &'a str) -> Result<DataUri<'a>, ()> {
    let uri = uri.strip_prefix("data:").ok_or(())?;
    let (mime_type, data) = split_once(uri, ',').ok_or(())?;

    let (mime_type, base64) = match mime_type.strip_suffix(";base64") {
      Some(mime_type) => (mime_type, true),
      None => (mime_type, false),
    };

    Ok(DataUri {
      mime_type,
      base64,
      data,
    })
  }

  fn decode(&self) -> Result<Vec<u8>, base64::DecodeError> {
    if self.base64 {
      base64::decode(self.data)
    } else {
      Ok(self.data.as_bytes().to_owned())
    }
  }
}

async fn load_buffers(gltf: &Gltf, load_context: &LoadContext<'_>, asset_path: &Path) -> Option<Vec<Vec<u8>>> {
  const VALID_MIME_TYPES: &[&str] = &["application/octet-stream", "application/gltf-buffer"];

  let mut buffer_data = Vec::new();
  for buffer in gltf.buffers() {
    match buffer.source() {
      Source::Uri(uri) => {
        let uri = percent_encoding::percent_decode_str(uri).decode_utf8().unwrap();
        let uri = uri.as_ref();
        let buffer_bytes = match DataUri::parse(uri) {
          Ok(data_uri) if VALID_MIME_TYPES.contains(&data_uri.mime_type) => data_uri.decode().unwrap(),
          Ok(_) => return None,
          Err(()) => {
            // TODO: Remove this and add dep
            let buffer_path = asset_path.parent().unwrap().join(uri);
            let buffer_bytes = load_context.read_asset_bytes(buffer_path).await.unwrap();
            buffer_bytes
          }
        };
        buffer_data.push(buffer_bytes);
      }
      Source::Bin => {
        if let Some(blob) = gltf.blob.as_deref() {
          buffer_data.push(blob.into());
        } else {
          return None;
        }
      }
    }
  }

  Some(buffer_data)
}

impl AssetLoader for GltfLoaderII {
  fn load<'a>(
    &'a self,
    bytes: &'a [u8],
    load_context: &'a mut LoadContext,
  ) -> BoxedFuture<'a, anyhow::Result<(), anyhow::Error>> {
    Box::pin(async {
      let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
      let gltf = Gltf::from_slice(bytes).unwrap();
      let buffers = load_buffers(&gltf, load_context, load_context.path()).await.unwrap();
      let mesh_map = gltf
        .document
        .meshes()
        .map(|x| (x.name().unwrap(), x))
        .collect::<HashMap<_, _>>();
      let mesh_render = mesh_map.get("Render").unwrap();
      for p in mesh_render.primitives() {
        let reader = p.reader(|r| Some(&buffers[r.index()]));
        let positions = reader.read_positions().unwrap().collect();
        let tex_coords = reader.read_tex_coords(0).unwrap().into_f32().collect();
        let indicies = reader.read_indices().unwrap().into_u32().collect();
        mesh.insert_attribute(
          MeshVertexAttribute::new("POSITIONS", 0, VertexFormat::Float32x3),
          VertexAttributeValues::Float32x3(positions),
        );
        mesh.insert_attribute(
          MeshVertexAttribute::new("TEXCOORD", 1, VertexFormat::Float32x2),
          VertexAttributeValues::Float32x2(tex_coords),
        );
        mesh.set_indices(Some(Indices::U32(indicies)));
      }
      load_context.set_default_asset(LoadedAsset::new(mesh));
      Ok(())
    })
  }

  fn extensions(&self) -> &[&str] {
    &["glb", "gltf"]
  }
}

#[derive(EnumIter, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Meshes {
  TestModel,
}

#[derive(Deref)]
pub struct GltfMeshStorage(pub Option<Handle<Mesh>>);

impl FromWorld for GltfMeshStorage {
  fn from_world(world: &mut World) -> Self {
    let asset_server = world.get_resource::<AssetServer>().unwrap();
    let gltf: Handle<Mesh> = asset_server.load("meshes/meshes.glb");
    GltfMeshStorage(Some(gltf))
  }
}
