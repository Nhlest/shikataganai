use crate::ecs::plugins::rendering::inventory_pipeline::inventory_cache::{
  ExtractedItems, ItemRenderEntry, ItemRenderMap,
};
use crate::ecs::plugins::rendering::inventory_pipeline::node::Initialised;
use crate::ecs::plugins::rendering::inventory_pipeline::INVENTORY_OUTPUT_TEXTURE_WIDTH;
use bevy::prelude::*;
use bevy::render::Extract;

pub fn clear_rerender(mut extracted_items: ResMut<ExtractedItems>) {
  extracted_items.rerender = false;
}

pub fn clear_renderapp_extraction(mut commands: Commands, initialised: Option<Res<Initialised>>) {
  if initialised.is_some() {
    commands.remove_resource::<ItemRenderMap>();
  }
}

pub fn update_extracted_items(mut extracted_items: ResMut<ExtractedItems>) {
  if !extracted_items.requested.is_empty() {
    let ExtractedItems {
      rendered,
      requested,
      rerender,
    } = extracted_items.as_mut();
    rendered.0.retain(|_, entry| entry.has_been_requested);
    rendered.0.extend(requested.iter().map(|x| {
      (
        *x,
        ItemRenderEntry {
          coord: (0.0, 0.0),
          has_been_requested: false,
        },
      )
    }));
    let mut x = 0.0;
    let mut y = 0.0;
    rendered.0.iter_mut().for_each(|(_, entry)| {
      entry.has_been_requested = false;
      entry.coord = (x, y);
      x += 1.0 / INVENTORY_OUTPUT_TEXTURE_WIDTH;
      if x >= 1.0 {
        x = 0.0;
        y += 1.0 / INVENTORY_OUTPUT_TEXTURE_WIDTH;
      }
    });

    dbg!(&rendered.0);

    *rerender = true;
    extracted_items.clear();
  }
}

pub fn extract_inventory_tiles(mut commands: Commands, extracted_items: Extract<Res<ExtractedItems>>) {
  if extracted_items.rerender {
    commands.insert_resource(extracted_items.rendered.clone());
  }
}
