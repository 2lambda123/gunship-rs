extern crate bootstrap_rs as bootstrap;
extern crate gl_util as gl;
extern crate parse_obj;

use bootstrap::window::*;
use gl::*;
use gl::context::Context;
use parse_obj::Obj;

fn main() {
    // Load mesh file and normalize indices for OpenGL.
    let obj = Obj::from_file("examples/epps_head.obj").unwrap();
    let mut raw_indices = Vec::new();
    for face in obj.position_indices() {
        for index in face {
            raw_indices.push(*index as u32);
        }
    }

    let mut window = Window::new("gl-util - wireframe example").unwrap();
    let context = Context::from_window(&window).unwrap();

    let mut vertex_buffer = VertexBuffer::new(&context);
    vertex_buffer.set_data_f32(obj.raw_positions());
    vertex_buffer.set_attrib_f32("position", AttribLayout { elements: 4, offset: 0, stride: 0 });

    let mut index_buffer = IndexBuffer::new(&context);
    index_buffer.set_data_u32(&*raw_indices);

    let vertex_array = VertexArray::with_index_buffer(&context, vertex_buffer, index_buffer);

    'outer: loop {
        while let Some(message) = window.next_message() {
            match message {
                Message::Close => break 'outer,
                _ => {},
            }
        }

        context.clear();
        DrawBuilder::new(&context, &vertex_array, DrawMode::Triangles)
            .map_attrib_location("position", AttributeLocation::from_index(0))
            .polygon_mode(PolygonMode::Line)
            .draw();
        context.swap_buffers();
    }
}
