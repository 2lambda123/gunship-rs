use std::rc::Rc;
use std::ptr;

use math::*;
use polygon::Camera;
use polygon::gl_render::{GLRender, ShaderProgram, GLMeshData};
use polygon::geometry::Mesh;
use resource::ResourceManager;

static mut instance: *mut DebugDrawInner = 0 as *mut DebugDrawInner;

#[derive(Debug)]
pub struct DebugDraw {
    renderer: Rc<GLRender>,

    shader: ShaderProgram,
    unit_cube: GLMeshData,

    inner: Box<DebugDrawInner>,
}

static CUBE_VERTS: [f32; 32] =
    [ 0.5,  0.5,  0.5, 1.0,
      0.5,  0.5, -0.5, 1.0,
      0.5, -0.5,  0.5, 1.0,
      0.5, -0.5, -0.5, 1.0,
     -0.5,  0.5,  0.5, 1.0,
     -0.5,  0.5, -0.5, 1.0,
     -0.5, -0.5,  0.5, 1.0,
     -0.5, -0.5, -0.5, 1.0,];
static CUBE_INDICES: [u32; 24] =
    [0, 1,
     1, 3,
     3, 2,
     2, 0,
     4, 5,
     5, 7,
     7, 6,
     6, 4,
     0, 4,
     1, 5,
     2, 6,
     3, 7,];

impl DebugDraw {
    pub fn new(renderer: Rc<GLRender>, resource_manager: &ResourceManager) -> DebugDraw {
        assert!(unsafe { instance.is_null() }, "Cannot create more than one instance of DebugDraw at a time");

        let mut inner = Box::new(DebugDrawInner {
            command_buffer: Vec::new(),
        });

        unsafe {
            instance = &mut *inner;
        }

        DebugDraw {
            renderer: renderer.clone(),

            shader: resource_manager.get_shader("shaders/debug_draw.glsl").unwrap(),
            unit_cube: build_mesh(&*renderer, &CUBE_VERTS, &CUBE_INDICES),

            inner: inner,
        }
    }

    pub fn flush_commands(&mut self, camera: &Camera) {
        for command in &self.inner.command_buffer {
            match command {
                &DebugDrawCommand::Line { start, end } => {
                    self.renderer.draw_line(camera, &self.shader, start, end);
                },
                &DebugDrawCommand::Box { center, widths } => {
                    let model_transform =
                        Matrix4::from_point(center) * Matrix4::from_scale_vector(widths);
                    self.renderer.draw_wireframe(camera, &self.shader, &self.unit_cube, model_transform);
                }
            }
        }

        self.inner.command_buffer.clear();
    }
}

impl Drop for DebugDraw {
    fn drop(&mut self) {
        unsafe {
            instance = ptr::null_mut();
        }
    }
}

/// Creates a mesh from a list of vertices and indices.
fn build_mesh(renderer: &GLRender, vertices: &[f32], indices: &[u32]) -> GLMeshData {
    let mesh = Mesh::from_raw_data(vertices, indices);
    renderer.gen_mesh(&mesh)
}

#[derive(Debug, Clone)]
pub enum DebugDrawCommand {
    Line {
        start: Point,
        end: Point,
    },
    Box {
        center: Point,
        widths: Vector3,
    }
}

#[derive(Debug)]
struct DebugDrawInner {
    command_buffer: Vec<DebugDrawCommand>,
}

pub fn draw_command(command: DebugDrawCommand) {
    debug_assert!(unsafe { !instance.is_null() }, "Cannot use debug drawing if there is no instance");

    let inner = unsafe { &mut *instance };
    inner.command_buffer.push(command);
}

pub fn draw_line(start: Point, end: Point) {
    draw_command(DebugDrawCommand::Line {
        start: start,
        end: end,
    });
}

pub fn draw_box_min_max(min: Point, max: Point) {
    let diff = max - min;
    draw_command(DebugDrawCommand::Box {
        center: min + diff  * 0.5,
        widths: diff,
    });
}

pub fn draw_box_center_widths(center: Point, widths: Vector3) {
    draw_command(DebugDrawCommand::Box {
        center: center,
        widths: widths,
    });
}