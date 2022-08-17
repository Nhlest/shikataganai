use std::io::Cursor;
use std::sync::{Arc, Mutex};

use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::input::{ButtonState, InputSystem};
use bevy::prelude::*;
use bevy::render::render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext, SlotInfo, SlotType};
use bevy::render::renderer::{RenderContext, RenderDevice, RenderQueue};
use bevy::render::view::ExtractedWindows;
use bevy::render::RenderApp;
use bevy::window::{WindowResized, WindowScaleFactorChanged};
use bevy::winit::WinitWindows;
use image::GenericImageView;
use imgui::{Context, FontId, FontSource, TextureId, Ui};
use imgui_wgpu::Texture as ImguiTexture;
use imgui_wgpu::{Renderer, RendererConfig, TextureConfig};
use imgui_winit_support::WinitPlatform;
use wgpu::TextureFormat::Bgra8UnormSrgb;
use wgpu::{BindGroupDescriptor, BindGroupEntry, BindingResource, TextureDimension, TextureUsages};
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event::*;

pub static mut IMGUI_CTX: Option<Context> = None;
pub static mut IMGUI_UI: Option<Ui> = None;

pub struct GUITextureAtlas(pub TextureId);

pub struct ImguiPlugin;

pub struct ImguiState;

impl !Send for ImguiState {}
impl !Sync for ImguiState {}

pub struct SmallFont(pub FontId);
pub struct BigFont(pub FontId);

pub const IMGUI_PASS: &'static str = "Imgui Pass";
pub const TEXTURE_NODE_INPUT_SLOT: &'static str = "Texture Slot Input";

impl ImguiState {
  pub fn get_current_frame<'a>(&self) -> &'a mut Ui<'static> {
    unsafe { IMGUI_UI.as_mut().unwrap() }
  }
  pub fn get_current_context<'a>(&self) -> &'a mut Context {
    unsafe { IMGUI_CTX.as_mut().unwrap() }
  }
}

fn start_frame(
  mut platform: NonSendMut<WinitPlatform>,
  // mut ev_cursor_entered: EventReader<CursorEntered>,
  // mut ev_cursor_left: EventReader<CursorLeft>,
  mut ev_cursor: EventReader<CursorMoved>,
  mut ev_mouse_button_input: EventReader<MouseButtonInput>,
  mut ev_mouse_wheel: EventReader<MouseWheel>,
  mut ev_resized: EventReader<WindowResized>,
  mut ev_scale: EventReader<WindowScaleFactorChanged>,
  // mut ev_received_character: EventReader<ReceivedCharacter>,
  // mut ev_window_focused: EventReader<WindowFocused>,
  // mut ev_window_created: EventReader<WindowCreated>,
  mut ev_keyboard_input: EventReader<KeyboardInput>,
  mut ev_received_character: EventReader<ReceivedCharacter>,
  windows: Res<Windows>,
  winit_windows: NonSend<WinitWindows>,
) {
  unsafe {
    let ctx = IMGUI_CTX.as_mut().unwrap();

    if windows.get_primary().is_none() {
      return;
    }

    let w_id = windows.get_primary().unwrap().id();
    let height = windows.get_primary().unwrap().height();
    let window = winit_windows.get_window(w_id).unwrap();
    let mut size = window.inner_size();

    for i in ev_scale.iter() {
      let event: Event<()> = Event::WindowEvent {
        window_id: window.id(),
        event: WindowEvent::ScaleFactorChanged {
          scale_factor: i.scale_factor,
          new_inner_size: &mut size,
        },
      };
      platform.handle_event(ctx.io_mut(), window, &event);
    }

    for i in ev_resized.iter() {
      let event: Event<()> = Event::WindowEvent {
        window_id: window.id(),
        event: WindowEvent::Resized(PhysicalSize::from_logical(
          LogicalSize::new(i.width, i.height),
          window.scale_factor(),
        )),
      };
      platform.handle_event(ctx.io_mut(), window, &event);
    }

    for i in ev_cursor.iter() {
      let CursorMoved { position, .. } = i;
      let p: PhysicalPosition<f64> = PhysicalPosition {
        x: position.x as f64 * window.scale_factor(),
        y: (height - position.y) as f64 * window.scale_factor(),
      };
      let event: Event<()> = Event::WindowEvent {
        window_id: window.id(),
        event: WindowEvent::CursorMoved {
          device_id: DeviceId::dummy(),
          position: p,
          modifiers: Default::default(),
        },
      };
      platform.handle_event(ctx.io_mut(), window, &event);
    }
    for i in ev_mouse_wheel.iter() {
      let MouseWheel { x, y, .. } = i;
      let event: Event<()> = Event::WindowEvent {
        window_id: window.id(),
        event: WindowEvent::MouseWheel {
          device_id: DeviceId::dummy(),
          delta: MouseScrollDelta::PixelDelta(PhysicalPosition::new(*x as f64, *y as f64)),
          phase: TouchPhase::Moved,
          modifiers: Default::default(),
        },
      };
      platform.handle_event(ctx.io_mut(), window, &event);
    }
    for i in ev_mouse_button_input.iter() {
      let MouseButtonInput { button, state } = i;
      let state = match state {
        ButtonState::Pressed => ElementState::Pressed,
        ButtonState::Released => ElementState::Released,
      };
      let button = match button {
        bevy::input::mouse::MouseButton::Left => winit::event::MouseButton::Left,
        bevy::input::mouse::MouseButton::Right => winit::event::MouseButton::Right,
        bevy::input::mouse::MouseButton::Middle => winit::event::MouseButton::Middle,
        bevy::input::mouse::MouseButton::Other(a) => winit::event::MouseButton::Other(*a),
      };
      let event: Event<()> = Event::WindowEvent {
        window_id: window.id(),
        event: WindowEvent::MouseInput {
          device_id: DeviceId::dummy(),
          state,
          button,
          modifiers: Default::default(),
        },
      };
      platform.handle_event(ctx.io_mut(), window, &event);
    }
    for i in ev_keyboard_input.iter() {
      let event: Event<()> = Event::WindowEvent {
        window_id: window.id(),
        event: WindowEvent::KeyboardInput {
          device_id: DeviceId::dummy(),
          input: winit::event::KeyboardInput {
            scancode: i.scan_code,
            state: match i.state {
              ButtonState::Pressed => ElementState::Pressed,
              ButtonState::Released => ElementState::Released,
            },
            virtual_keycode: i.key_code.map(|x| std::mem::transmute(x as u32)),
            modifiers: Default::default(),
          },
          is_synthetic: false,
        },
      };
      platform.handle_event(ctx.io_mut(), window, &event);
    }
    for i in ev_received_character.iter() {
      let event: Event<()> = Event::WindowEvent {
        window_id: window.id(),
        event: WindowEvent::ReceivedCharacter(i.char),
      };
      platform.handle_event(ctx.io_mut(), window, &event);
    }
    platform
      .prepare_frame(ctx.io_mut(), window)
      .expect("Failed to prepare frame");
    IMGUI_UI = Some(ctx.frame());
  }
}
#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemLabel)]
pub struct ImguiFrameSystem;

impl Plugin for ImguiPlugin {
  fn build(&self, app: &mut App) {
    app.insert_non_send_resource(ImguiState);
    app.add_system_to_stage(CoreStage::PreUpdate, start_frame.after(InputSystem));
    let windows = app.world.get_resource::<Windows>().unwrap();
    let winit_windows = app.world.get_non_send_resource::<WinitWindows>().unwrap();
    let window = winit_windows.get_window(windows.get_primary().unwrap().id()).unwrap();
    let device = app.world.get_resource::<RenderDevice>().unwrap().wgpu_device();
    let q = app.world.get_resource::<RenderQueue>().unwrap().clone();
    let queue = q.as_ref();

    let hidpi_factor = window.scale_factor();
    let mut imgui = Context::create();

    let mut platform = WinitPlatform::init(&mut imgui);
    platform.attach_window(imgui.io_mut(), window, imgui_winit_support::HiDpiMode::Default);

    imgui.set_ini_filename(None);

    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    let font_size = (13.0 * hidpi_factor) as f32;
    let smol_font = imgui.fonts().add_font(&[FontSource::DefaultFontData {
      config: Some(imgui::FontConfig {
        oversample_h: 1,
        pixel_snap_h: true,
        size_pixels: font_size,
        ..Default::default()
      }),
    }]);

    let font_size = (21.0 * hidpi_factor) as f32;
    let big_font = imgui.fonts().add_font(&[FontSource::DefaultFontData {
      config: Some(imgui::FontConfig {
        oversample_h: 2,
        oversample_v: 2,
        pixel_snap_h: true,
        size_pixels: font_size,
        ..Default::default()
      }),
    }]);

    let renderer_config = RendererConfig {
      texture_format: Bgra8UnormSrgb,
      ..Default::default()
    };

    let mut renderer = Renderer::new(&mut imgui, device, queue, renderer_config);

    let diffuse_image = image::io::Reader::new(Cursor::new(include_bytes!("../../../assets/gui.png")))
      .with_guessed_format()
      .unwrap()
      .decode()
      .unwrap();
    let diffuse_rgba = diffuse_image.as_rgba8().unwrap();
    let dimensions = diffuse_image.dimensions();
    let texture_size = wgpu::Extent3d {
      width: dimensions.0,
      height: dimensions.1,
      depth_or_array_layers: 1,
    };

    let texture = ImguiTexture::new(
      &device,
      &renderer,
      TextureConfig {
        size: texture_size,
        label: Some("Imgui Texture"),
        format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        sampler_desc: wgpu::SamplerDescriptor {
          address_mode_u: wgpu::AddressMode::ClampToEdge,
          address_mode_v: wgpu::AddressMode::ClampToEdge,
          address_mode_w: wgpu::AddressMode::ClampToEdge,
          mag_filter: wgpu::FilterMode::Nearest,
          min_filter: wgpu::FilterMode::Nearest,
          mipmap_filter: wgpu::FilterMode::Nearest,
          ..Default::default()
        },
      },
    );

    texture.write(queue, diffuse_rgba.as_ref(), texture_size.width, texture_size.height);
    let tex_id = renderer.textures.insert(texture);
    app.insert_resource(GUITextureAtlas(tex_id));
    app
      .get_sub_app_mut(RenderApp)
      .unwrap()
      .insert_resource(GUITextureAtlas(tex_id));

    app.insert_non_send_resource(SmallFont(smol_font));
    app.insert_non_send_resource(BigFont(big_font));

    //TODO: Save this
    // let mut last_frame = Instant::now();
    // let mut last_cursor = None;

    unsafe { IMGUI_CTX = Some(imgui) };

    app.insert_non_send_resource(platform);

    if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
      let mut render_graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();
      render_graph.add_node(IMGUI_PASS, ImguiNode::new(renderer));
    }
  }
}

pub struct ImguiNode {
  pub renderer: Arc<Mutex<Renderer>>,
}

impl ImguiNode {
  pub fn new(renderer: Renderer) -> Self {
    ImguiNode {
      renderer: Arc::new(Mutex::new(renderer)),
    }
  }
}

impl Node for ImguiNode {
  fn update(&mut self, _world: &mut World) {}

  fn input(&self) -> Vec<SlotInfo> {
    vec![SlotInfo {
      name: TEXTURE_NODE_INPUT_SLOT.into(),
      slot_type: SlotType::TextureView,
    }]
  }

  fn run(
    &self,
    graph: &mut RenderGraphContext,
    render_context: &mut RenderContext,
    world: &World,
  ) -> Result<(), NodeRunError> {
    // TODO: blinking in inventory. IMGUI uses old texture for a single frame for some reason.
    let inventory_texture = graph.get_input_texture(TEXTURE_NODE_INPUT_SLOT).unwrap();

    let q = self.renderer.clone();
    let mut renderer = q.lock().unwrap();

    let tex_id = world.resource::<GUITextureAtlas>().0;

    let sampler = render_context
      .render_device
      .wgpu_device()
      .create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
      });

    let bind_group = render_context
      .render_device
      .wgpu_device()
      .create_bind_group(&BindGroupDescriptor {
        label: Some("Inventory Texture Bind Group"),
        layout: &renderer.texture_layout,
        entries: &[
          BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(&inventory_texture),
          },
          BindGroupEntry {
            binding: 1,
            resource: BindingResource::Sampler(&sampler),
          },
        ],
      });

    let tex = renderer.textures.get_mut(tex_id).unwrap();
    tex.bind_group = bind_group;

    for (_, extracted_window) in &world.get_resource::<ExtractedWindows>().unwrap().windows {
      // TODO: save window id
      let ui = unsafe { std::mem::take(&mut IMGUI_UI) }.unwrap();
      let swap_chain_texture = extracted_window.swap_chain_texture.as_ref().unwrap().clone();

      let mut rpass = render_context
        .command_encoder
        .begin_render_pass(&wgpu::RenderPassDescriptor {
          label: None,
          color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &swap_chain_texture,
            resolve_target: None,
            ops: wgpu::Operations {
              load: wgpu::LoadOp::Load,
              store: true,
            },
          })],
          depth_stencil_attachment: None,
        });

      renderer
        .render(
          ui.render(),
          world.get_resource::<RenderQueue>().unwrap(),
          render_context.render_device.wgpu_device(),
          &mut rpass,
        )
        .unwrap();
    }
    Ok(())
  }
}
