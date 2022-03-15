extern crate core;

use egui::{Align2, ComboBox, Ui};
use ultraviolet::{Isometry3, Rotor3, UVec3, Vec3, Vec4};
use wgpu::{BindGroup, CommandBuffer, Features, IndexFormat, RenderPipeline, ShaderStages};

use app::{GuiFrame, Options, Os};
use common::input::RawInput;
use common::screen::ScreenSize;
use gfx::{Frame, Gfx, include_shader};
use gfx::bind_group::CombinedBindGroupLayoutBuilder;
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::camera::{Camera, CameraInput};
use gfx::debug_renderer::{DebugRenderer, PointVertex, RegularVertex};
use gfx::display_math::UVec3DisplayExt;
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;
use gfx::texture::{GfxTexture, TextureBuilder};
use gui_widget::UiWidgetsExt;
use voxel_meshing::chunk::{ChunkSampleArray, ChunkSamples, ChunkSize, ChunkVertices, GenericChunkSize, Vertex};
use voxel_meshing::marching_cubes::{self, MarchingCubes, RegularCell};
use voxel_meshing::uniform::{CameraUniform, LightSettings, ModelUniform};

pub struct MarchingCubesDemo {
  camera: Camera,
  debug_renderer: DebugRenderer,

  camera_uniform: CameraUniform,
  light_settings: LightSettings,
  model_uniform: ModelUniform,

  camera_uniform_buffer: GfxBuffer,
  light_uniform_buffer: GfxBuffer,
  _model_uniform_buffer: GfxBuffer,
  uniform_bind_group: BindGroup,
  depth_texture: GfxTexture,
  render_pipeline: RenderPipeline,
  multisampled_framebuffer: GfxTexture,

  chunk_samples: ChunkSamples<C>,
  gui_equivalence_class: u8,
}

#[derive(Default)]
pub struct Input {
  camera: CameraInput,
}

type C = GenericChunkSize<1>;
type MC = MarchingCubes<C>;

const SAMPLE_COUNT: u32 = 4;

impl app::Application for MarchingCubesDemo {
  fn new(os: &Os, gfx: &Gfx) -> Self {
    let viewport = os.window.get_inner_size().physical;

    let mut camera = Camera::with_defaults_arcball_orthographic(viewport);
    camera.arcball.distance = -2.0;
    let debug_renderer = DebugRenderer::new(gfx, SAMPLE_COUNT, camera.get_view_projection_matrix());

    let camera_uniform = CameraUniform::from_camera_sys(&camera);
    let mut light_settings = LightSettings::default();
    light_settings.uniform.ambient = 0.2;
    light_settings.uniform.color = Vec3::new(0.0, 0.5, 0.35);
    let extends = C::CELLS_IN_CHUNK_ROW as f32 / 2.0;
    let transform = Isometry3::new(Vec3::new(-extends, -extends, -extends), Rotor3::identity());
    let model_uniform = ModelUniform::from_transform(transform);

    let depth_texture = TextureBuilder::new_depth_32_float(viewport)
      .with_sample_count(SAMPLE_COUNT)
      .build(&gfx.device);

    let vertex_shader_module = gfx.device.create_shader_module(&include_shader!("vert"));
    let fragment_shader_module = gfx.device.create_shader_module(&include_shader!("frag"));

    let camera_uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Camera uniform buffer")
      .build_with_data(&gfx.device, &[camera_uniform]);
    let (camera_uniform_bind_group_layout_entry, camera_uniform_bind_group_entry) =
      camera_uniform_buffer.create_uniform_binding_entries(0, ShaderStages::VERTEX_FRAGMENT);
    let light_uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Light uniform buffer")
      .build_with_data(&gfx.device, &[light_settings.uniform]);
    let (light_uniform_bind_group_layout_entry, light_uniform_bind_group_entry) =
      light_uniform_buffer.create_uniform_binding_entries(1, ShaderStages::FRAGMENT);
    let model_uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Model uniform buffer")
      .build_with_data(&gfx.device, &[model_uniform]);
    let (model_uniform_bind_group_layout_entry, model_uniform_bind_group_entry) =
      model_uniform_buffer.create_uniform_binding_entries(2, ShaderStages::VERTEX);
    let (uniform_bind_group_layout, uniform_bind_group) = CombinedBindGroupLayoutBuilder::new()
      .with_layout_entries(&[camera_uniform_bind_group_layout_entry, light_uniform_bind_group_layout_entry, model_uniform_bind_group_layout_entry])
      .with_entries(&[camera_uniform_bind_group_entry, light_uniform_bind_group_entry, model_uniform_bind_group_entry])
      .with_layout_label("Marching cubes uniform bind group layout")
      .with_label("Marching cubes uniform bind group")
      .build(&gfx.device);

    let (_, render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_bind_group_layouts(&[&uniform_bind_group_layout])
      .with_default_fragment_state(&fragment_shader_module, &gfx.surface)
      .with_vertex_buffer_layouts(&[Vertex::buffer_layout()])
      //.with_cull_mode(Some(Face::Back))
      .with_multisample_count(SAMPLE_COUNT)
      .with_depth_texture(depth_texture.format)
      .with_layout_label("Marching cubes pipeline layout")
      .with_label("Marching cubes render pipeline")
      .build(&gfx.device);
    let multisampled_framebuffer = TextureBuilder::new_multisampled_framebuffer(&gfx.surface, SAMPLE_COUNT)
      .with_texture_label("Multisampling texture")
      .with_texture_view_label("Multisampling texture view")
      .build(&gfx.device);

    let chunk_samples = ChunkSamples::Mixed(ChunkSampleArray::<C>::new_with(0.0));

    Self {
      camera,
      debug_renderer,

      camera_uniform,
      light_settings,
      model_uniform,

      camera_uniform_buffer,
      light_uniform_buffer,
      _model_uniform_buffer: model_uniform_buffer,
      uniform_bind_group,
      depth_texture,
      render_pipeline,
      multisampled_framebuffer,

      chunk_samples,
      gui_equivalence_class: 0,
    }
  }

  fn screen_resize(&mut self, _os: &Os, gfx: &Gfx, screen_size: ScreenSize) {
    let viewport = screen_size.physical;
    self.camera.viewport = viewport;
    self.depth_texture = TextureBuilder::new_depth_32_float(viewport)
      .with_sample_count(SAMPLE_COUNT)
      .build(&gfx.device);
    self.multisampled_framebuffer = TextureBuilder::new_multisampled_framebuffer(&gfx.surface, SAMPLE_COUNT)
      .build(&gfx.device);
  }

  type Input = Input;

  fn process_input(&mut self, input: RawInput) -> Input {
    let camera = CameraInput::from(&input);
    Input { camera }
  }

  fn add_to_debug_menu(&mut self, ui: &mut Ui) {
    ui.checkbox(&mut self.camera.show_debug_gui, "Camera");
  }

  fn render<'a>(&mut self, _os: &Os, gfx: &Gfx, mut frame: Frame<'a>, gui_frame: &GuiFrame, input: &Self::Input) -> Box<dyn Iterator<Item=CommandBuffer>> {
    // Update camera
    self.camera.update(&input.camera, frame.time.delta, &gui_frame);
    self.camera_uniform.update_from_camera_sys(&self.camera);

    // Debug GUI
    egui::Window::new("Marching Cubes")
      .anchor(Align2::LEFT_TOP, egui::Vec2::default())
      .show(&gui_frame, |ui| {
        self.light_settings.render_gui(ui);
        let samples = if let ChunkSamples::Mixed(chunk_samples_array) = &mut self.chunk_samples {
          chunk_samples_array
        } else {
          panic!();
        };
        ui.collapsing_open("Cell", |ui| {
          ui.horizontal(|ui| {
            ComboBox::from_id_source("Equivalence class")
              .selected_text(format!("{:?}", self.gui_equivalence_class))
              .show_ui(ui, |ui| {
                for i in 0..18 {
                  ui.selectable_value(&mut self.gui_equivalence_class, i, format!("{:?}", i));
                }
              });
            if ui.button("Set").clicked() {
              let inside = -1.0;
              match self.gui_equivalence_class {
                0 => {
                  samples.set_all_to(1.0);
                }
                1 => {
                  samples.set_all_to(1.0);
                  samples.set(0, 0, 1, inside);
                }
                2 => {
                  samples.set_all_to(1.0);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 1, 1, inside);
                }
                3 => {
                  samples.set_all_to(1.0);
                  samples.set(0, 0, 0, inside);
                  samples.set(0, 0, 1, inside);
                }
                4 => {
                  samples.set_all_to(1.0);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 1, 0, inside);
                }
                5 => {
                  samples.set_all_to(1.0);
                  samples.set(1, 0, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 0, 1, inside);
                }
                6 => {
                  samples.set_all_to(1.0);
                  samples.set(0, 0, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 1, 1, inside);
                }
                7 => {
                  samples.set_all_to(1.0);
                  samples.set(0, 1, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 1, 1, inside);
                }
                8 => {
                  samples.set_all_to(1.0);
                  samples.set(1, 0, 0, inside);
                  samples.set(0, 1, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 0, 1, inside);
                }
                9 => {
                  samples.set_all_to(1.0);
                  samples.set(0, 0, 0, inside);
                  samples.set(1, 1, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 1, 1, inside);
                }
                10 => {
                  samples.set_all_to(1.0);
                  samples.set(1, 0, 0, inside);
                  samples.set(0, 1, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 1, 1, inside);
                }
                11 => {
                  samples.set_all_to(1.0);
                  samples.set(0, 0, 0, inside);
                  samples.set(1, 0, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 0, 1, inside);
                }
                12 => {
                  samples.set_all_to(1.0);
                  samples.set(1, 0, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 0, 1, inside);
                  samples.set(0, 1, 1, inside);
                }
                13 => {
                  samples.set_all_to(1.0);
                  samples.set(0, 0, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 0, 1, inside);
                  samples.set(1, 1, 1, inside);
                }
                14 => {
                  samples.set_all_to(1.0);
                  samples.set(1, 0, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 0, 1, inside);
                  samples.set(1, 1, 1, inside);
                }
                15 => {
                  samples.set_all_to(1.0);
                  samples.set(1, 0, 0, inside);
                  samples.set(0, 1, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 0, 1, inside);
                  samples.set(0, 1, 1, inside);
                  samples.set(1, 1, 1, inside);
                }
                16 => {
                  samples.set_all_to(1.0);
                  samples.set(1, 0, 0, inside);
                  samples.set(0, 1, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 0, 1, inside);
                  samples.set(0, 1, 1, inside);
                }
                17 => {
                  samples.set_all_to(1.0);
                  samples.set(1, 0, 0, inside);
                  samples.set(0, 1, 0, inside);
                  samples.set(0, 0, 1, inside);
                  samples.set(1, 0, 1, inside);
                  samples.set(1, 1, 1, inside);
                }
                _ => {}
              }
            }
          });
          ui.horizontal(|ui| {
            if ui.button("Flip").clicked() {
              samples.flip_all();
            }
            if ui.button("To +0.0").clicked() {
              samples.set_all_to(0.0);
            }
            if ui.button("To -0.0").clicked() {
              samples.set_all_to(-0.0);
            }
            if ui.button("To +1.0").clicked() {
              samples.set_all_to(1.0);
            }
            if ui.button("To -1.0").clicked() {
              samples.set_all_to(-1.0);
            }
            ui.end_row();
          });
        });

        let local_coordinates = MC::local_coordinates(RegularCell::new(0, 0, 0));
        let global_coordinates = MC::global_coordinates(UVec3::zero(), 1, &local_coordinates);
        let values = MC::sample(samples, &local_coordinates);
        let case = MarchingCubes::<C>::case(&values);
        let cell_class = marching_cubes::tables::REGULAR_CELL_CLASS[case as usize] as usize;
        let triangulation_info = marching_cubes::tables::REGULAR_CELL_DATA[cell_class];
        let vertex_count = triangulation_info.get_vertex_count() as usize;
        let triangle_count = triangulation_info.get_triangle_count() as usize;
        let vertices_data = marching_cubes::tables::REGULAR_VERTEX_DATA[case as usize];

        ui.collapsing_open_with_grid("Data", "Data Grid", |ui| {
          ui.label("Case");
          ui.monospace(format!("{case} (class: {cell_class})"));
          ui.end_row();
          ui.label("Counts");
          ui.monospace(format!("Vertices: {vertex_count}, triangles: {triangle_count}"));
          ui.end_row();
        });

        ui.collapsing_open_with_grid("Voxels", "Voxels Grid", |ui| {
          ui.label("#");
          ui.label("local");
          ui.label("global");
          ui.label("value");
          ui.end_row();
          for i in 0..8 {
            let local = local_coordinates[i];
            ui.monospace(format!("{}", i));
            ui.monospace(format!("{}", local.display()));
            ui.monospace(format!("{}", global_coordinates[i].display()));
            let value = samples.sample_mut(local);
            let response = ui.drag("", value, 0.01);
            if response.secondary_clicked() {
              *value *= -1.0;
            }
            if response.middle_clicked() {
              *value = 0.0;
            }
            ui.end_row();
          }
        });

        ui.collapsing_open_with_grid("Vertices", "Vertices", |ui| {
          ui.label("#");
          ui.label("-x?");
          ui.label("-y?");
          ui.label("-z?");
          ui.label("new?");
          ui.label("vtx idx");
          ui.label("vox a idx");
          ui.label("vox b idx");
          ui.end_row();
          for (i, vd) in vertices_data[0..vertex_count].iter().enumerate() {
            ui.monospace(format!("{}", i));
            ui.monospace(format!("{}", vd.subtract_x()));
            ui.monospace(format!("{}", vd.subtract_y()));
            ui.monospace(format!("{}", vd.subtract_z()));
            ui.monospace(format!("{}", vd.new_vertex()));
            ui.monospace(format!("{}", vd.vertex_index()));
            ui.monospace(format!("{}", vd.voxel_a_index()));
            ui.monospace(format!("{}", vd.voxel_b_index()));
            ui.end_row();
          }
        });

        ui.collapsing_open_with_grid("Triangulation", "Triangulation Grid", |ui| {
          ui.label("#");
          ui.label("triangle idxs");
          ui.end_row();
          for i in (0..triangle_count * 3).step_by(3) {
            ui.monospace(format!("{}", i / 3));
            ui.monospace(format!("{} {} {}", triangulation_info.vertex_index[i + 0], triangulation_info.vertex_index[i + 1], triangulation_info.vertex_index[i + 2]));
            ui.end_row();
          }
        });
      });

    // Write uniforms
    self.camera_uniform_buffer.write_whole_data(&gfx.queue, &[self.camera_uniform]);
    self.light_uniform_buffer.write_whole_data(&gfx.queue, &[self.light_settings.uniform]);

    // Run marching cubes to create triangles from voxels
    let mut chunk_vertices = ChunkVertices::new();
    let marching_cubes = MC::new();
    marching_cubes.extract_chunk(UVec3::zero(), 1, &self.chunk_samples, &mut chunk_vertices);
    let vertex_buffer = BufferBuilder::new()
      .with_vertex_usage()
      .with_label("Voxel meshing vertex buffer")
      .build_with_data(&gfx.device, &chunk_vertices.vertices());
    let index_buffer = BufferBuilder::new()
      .with_index_usage()
      .with_label("Voxel meshing index buffer")
      .build_with_data(&gfx.device, &chunk_vertices.indices());
    let mut render_pass = RenderPassBuilder::new()
      .with_depth_texture(&self.depth_texture.view)
      .with_label("Marching cubes render pass")
      .begin_render_pass_for_multisampled_swap_chain_with_clear(frame.encoder, &self.multisampled_framebuffer.view, &frame.output_texture);
    render_pass.push_debug_group("Draw voxelized mesh");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
    render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    render_pass.draw_indexed(0..chunk_vertices.indices().len() as u32, 0, 0..1);
    render_pass.pop_debug_group();
    drop(render_pass);

    // Debug rendering
    self.debug_renderer.clear();
    let chunk_samples_array = if let ChunkSamples::Mixed(chunk_samples_array) = &self.chunk_samples {
      chunk_samples_array
    } else {
      panic!();
    };
    // Axes
    self.debug_renderer.draw_axes_lines(Vec3::one() * 0.5, 0.5);
    // Voxels
    for z in 0..C::VOXELS_IN_CHUNK_ROW {
      for y in 0..C::VOXELS_IN_CHUNK_ROW {
        for x in 0..C::VOXELS_IN_CHUNK_ROW {
          let position = UVec3::new(x, y, z);
          let sample = chunk_samples_array.sample(position);
          if sample.is_sign_negative() {
            self.debug_renderer.draw_point(position.into(), Vec4::new(1.0, 1.0, 1.0, 1.0), 20.0);
          }
        }
      }
    }
    // Cells
    for z in 0..C::CELLS_IN_CHUNK_ROW {
      for y in 0..C::CELLS_IN_CHUNK_ROW {
        for x in 0..C::CELLS_IN_CHUNK_ROW {
          let position = UVec3::new(x, y, z);
          self.debug_renderer.draw_cube_lines(position.into(), 1.0, Vec4::new(1.0, 1.0, 1.0, 1.0));
        }
      }
    }
    // Marching cubes wireframe and points
    self.debug_renderer.draw_triangle_vertices_wireframe_indexed(
      chunk_vertices.vertices().into_iter().map(|v| RegularVertex::new(v.position, Vec4::one())),
      chunk_vertices.indices().into_iter().copied(),
    );
    self.debug_renderer.draw_point_vertices(chunk_vertices.vertices().into_iter().map(|v| PointVertex::new(v.position, Vec4::one(), 10.0)));
    // Perform the actual debug rendering
    self.debug_renderer.render(gfx, &mut frame, Some(&self.multisampled_framebuffer), self.camera.get_view_projection_matrix() * self.model_uniform.model);

    Box::new(std::iter::empty())
  }
}

fn main() {
  app::run::<MarchingCubesDemo>(Options {
    name: "Marching cubes".to_string(),
    request_graphics_device_features: Features::empty() | DebugRenderer::request_features(),
    ..Options::default()
  }).unwrap();
}
