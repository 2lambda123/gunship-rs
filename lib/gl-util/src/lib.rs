//! Utility wrappers to simplify writing OpenGL code.
//!
//! This crate aspires to provide an abstraction over OpenGL's raw API in order to simplify the
//! task of writing higher-level rendering code for OpenGL. `gl-util` is much in the vein of
//! [glutin](https://github.com/tomaka/glium) and [gfx-rs](https://github.com/gfx-rs/gfx),
//! the main difference being that it is much more poorly constructed and is being developed by
//! someone much less OpenGL experience.

#![feature(associated_consts)]
#![feature(pub_restricted)]

extern crate bootstrap_rs as bootstrap;
extern crate bootstrap_gl as gl;

use context::{Context, ContextInner};
use gl::*;
use shader::Program;
use std::mem;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use texture::Texture2d;

pub use gl::{
    AttributeLocation,
    Comparison,
    DestFactor,
    DrawMode,
    Face,
    PolygonMode,
    ShaderType,
    SourceFactor,
    WindingOrder,
};

pub mod context;
pub mod shader;
pub mod texture;

#[cfg(target_os="windows")]
#[path="windows\\mod.rs"]
pub mod platform;

/// Represents a buffer of vertex data and the layout of that data.
///
/// Wraps a vertex buffer object and vertex array object into one struct.
#[derive(Debug)]
pub struct VertexBuffer {
    buffer_name: BufferName,
    len: usize,
    element_len: usize,
    attribs: HashMap<String, AttribLayout>,

    pub(crate) context: gl::Context,
}

impl VertexBuffer {
    /// Creates a new `VertexBuffer` object.
    pub fn new(context: &Context) -> VertexBuffer {
        let context = context.raw();

        let mut buffer_name = BufferName::null();
        unsafe {
            let _guard = ::context::ContextGuard::new(context);
            gl::gen_buffers(1, &mut buffer_name);
        }

        VertexBuffer {
            buffer_name: buffer_name,
            len: 0,
            element_len: 0,
            attribs: HashMap::new(),

            context: context,
        }
    }

    /// Fills the buffer with the contents of the data slice.
    pub fn set_data_f32(&mut self, data: &[f32]) {
        self.len = data.len();

        let data_ptr = data.as_ptr() as *const ();
        let byte_count = data.len() * mem::size_of::<f32>();

        unsafe {
            let _guard = ::context::ContextGuard::new(self.context);
            gl::bind_buffer(BufferTarget::Array, self.buffer_name);
            gl::buffer_data(
                BufferTarget::Array,
                byte_count as isize,
                data_ptr,
                BufferUsage::StaticDraw);
            gl::bind_buffer(BufferTarget::Array, BufferName::null());
        }
    }

    /// Specifies how the data for a particular vertex attribute is laid out in the buffer.
    ///
    /// `layout` specifies the layout of the vertex attributes. `AttribLayout` includes the three
    /// values that are needed to fully describe the attribute: The offset, the number of elements
    /// in the attrib, and the stride between elements.
    ///
    /// TODO: Include more details about how to describe the layout of attrib data.
    pub fn set_attrib_f32<T: Into<String>>(
        &mut self,
        attrib: T,
        layout: AttribLayout,
    ) {
        // Calculate the number of elements based on the attribute.
        // TODO: Verify that each attrib has the same element length.
        self.element_len = (self.len - layout.offset) / layout.elements + layout.stride;
        self.attribs.insert(attrib.into(), layout);
    }
}

impl Drop for VertexBuffer {
    fn drop(&mut self) {
        unsafe {
            let _guard = ::context::ContextGuard::new(self.context);
            gl::delete_buffers(1, &mut self.buffer_name);
        }
    }
}

/// Describes the layout of vertex data in a `VertexBuffer`.
///
/// See [`VertexBuffer::set_attrib_f32()`][VertexBuffer::set_attrib_f32] for more information.
///
/// [VertexBuffer::set_attrib_f32]: TODO: Figure out link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AttribLayout {
    pub elements: usize,
    pub stride: usize,
    pub offset: usize,
}

/// Represents a buffer of index data used to index into a `VertexBuffer` when drawing.
#[derive(Debug)]
pub struct IndexBuffer {
    buffer_name: BufferName,
    len: usize,

    pub(crate) context: gl::Context,
}

impl IndexBuffer {
    /// Creates a new index buffer.
    pub fn new(context: &Context) -> IndexBuffer {
        let context = context.raw();
        let mut buffer_name = BufferName::null();
        unsafe {
            let _guard = ::context::ContextGuard::new(context);
            gl::gen_buffers(1, &mut buffer_name);
        }

        IndexBuffer {
            buffer_name: buffer_name,
            len: 0,

            context: context,
        }
    }

    /// Fills the index buffer with the provided data.
    pub fn set_data_u32(&mut self, data: &[u32]) {
        self.len = data.len();

        let data_ptr = data.as_ptr() as *const ();
        let byte_count = data.len() * mem::size_of::<u32>();

        unsafe {
            let _guard = ::context::ContextGuard::new(self.context);
            gl::bind_buffer(BufferTarget::ElementArray, self.buffer_name);
            gl::buffer_data(
                BufferTarget::ElementArray,
                byte_count as isize,
                data_ptr,
                BufferUsage::StaticDraw);
            gl::bind_buffer(BufferTarget::ElementArray, BufferName::null());
        }
    }
}

impl Drop for IndexBuffer {
    fn drop(&mut self) {
        unsafe {
            let _guard = ::context::ContextGuard::new(self.context);
            gl::delete_buffers(1, &mut self.buffer_name);
        }
    }
}

#[derive(Debug)]
pub struct VertexArray {
    vertex_array_name: VertexArrayName,
    vertex_buffer: VertexBuffer,
    index_buffer: Option<IndexBuffer>,

    context: Rc<RefCell<ContextInner>>,
}

impl VertexArray {
    pub fn new(context: &Context, vertex_buffer: VertexBuffer) -> VertexArray {
        let mut vertex_array_name = VertexArrayName::null();
        let context_inner = context.inner();
        unsafe {
            let mut context = context_inner.borrow_mut();
            let _guard = ::context::ContextGuard::new(context.raw());
            gl::gen_vertex_arrays(1, &mut vertex_array_name);

            context.bind_vertex_array(vertex_array_name);
            gl::bind_buffer(BufferTarget::Array, vertex_buffer.buffer_name);
        }

        VertexArray {
            vertex_array_name: vertex_array_name,
            vertex_buffer: vertex_buffer,
            index_buffer: None,

            context: context_inner,
        }
    }

    pub fn with_index_buffer(context: &Context, vertex_buffer: VertexBuffer, index_buffer: IndexBuffer) -> VertexArray {
        let mut vertex_array_name = VertexArrayName::null();
        let context_inner = context.inner();
        unsafe {
            let mut context = context_inner.borrow_mut();
            let _guard = ::context::ContextGuard::new(context.raw());
            gl::gen_vertex_arrays(1, &mut vertex_array_name);

            context.bind_vertex_array(vertex_array_name);
            gl::bind_buffer(BufferTarget::Array, vertex_buffer.buffer_name);
            gl::bind_buffer(BufferTarget::ElementArray, index_buffer.buffer_name);
        }

        VertexArray {
            vertex_array_name: vertex_array_name,
            vertex_buffer: vertex_buffer,
            index_buffer: Some(index_buffer),

            context: context_inner,
        }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        let mut context = self.context.borrow_mut();
        let _guard = ::context::ContextGuard::new(context.raw());
        unsafe { gl::delete_vertex_arrays(1, &mut self.vertex_array_name); }
        context.unbind_vertex_array(self.vertex_array_name);
    }
}

/// A configuration object for specifying all of the various configurable options for a draw call.
// TODO: Change `DrawBuidler` to cull backfaces by default.
pub struct DrawBuilder<'a> {
    vertex_array: &'a VertexArray,
    draw_mode: DrawMode,
    polygon_mode: Option<PolygonMode>,
    program: Option<&'a Program>,
    cull: Option<Face>,
    depth_test: Option<Comparison>,
    winding_order: WindingOrder,
    blend: (SourceFactor, DestFactor),
    uniforms: HashMap<UniformLocation, UniformValue<'a>>,

    context: Rc<RefCell<ContextInner>>,
}

impl<'a> DrawBuilder<'a> {
    pub fn new(context: &Context, vertex_array: &'a VertexArray, draw_mode: DrawMode) -> DrawBuilder<'a> {
        // TODO: Make sure `vertex_array` comes from the right context.

        DrawBuilder {
            vertex_array: vertex_array,
            draw_mode: draw_mode,
            polygon_mode: None,
            program: None,
            cull: None,
            depth_test: None,
            winding_order: WindingOrder::default(),
            blend: Default::default(),
            uniforms: HashMap::new(),

            context: context.inner(),
        }
    }

    pub fn polygon_mode(&mut self, polygon_mode: PolygonMode) -> &mut DrawBuilder<'a> {
        self.polygon_mode = Some(polygon_mode);
        self
    }

    pub fn program(&mut self, program: &'a Program) -> &mut DrawBuilder<'a> {
        assert!(
            self.context.borrow().raw() == program.context,
            "Specified program's context does not match draw builder's context"
        );
        self.program = Some(program);
        self
    }

    pub fn cull(&mut self, face: Face) -> &mut DrawBuilder<'a> {
        self.cull = Some(face);
        self
    }

    pub fn depth_test(&mut self, comparison: Comparison) -> &mut DrawBuilder<'a> {
        self.depth_test = Some(comparison);
        self
    }

    pub fn winding(&mut self, winding_order: WindingOrder) -> &mut DrawBuilder<'a> {
        self.winding_order = winding_order;
        self
    }

    pub fn blend(
        &mut self,
        source_factor: SourceFactor,
        dest_factor: DestFactor
    ) -> &mut DrawBuilder<'a> {
        self.blend = (source_factor, dest_factor);
        self
    }

    /// Maps a vertex attribute to an attribute location for the current program.
    ///
    /// # Panics
    ///
    /// - If the the vertex buffer does not have an attribute named `buffer_attrib_name`.
    pub fn map_attrib_location(
        &mut self,
        buffer_attrib_name: &str,
        attrib_location: AttributeLocation
    ) -> &mut DrawBuilder<'a> {
        let layout = match self.vertex_array.vertex_buffer.attribs.get(buffer_attrib_name) {
            Some(&attrib_data) => attrib_data,
            None => panic!("Vertex buffer has no attribute \"{}\"", buffer_attrib_name),
        };

        unsafe {
            let mut context = self.context.borrow_mut();
            let _guard = ::context::ContextGuard::new(context.raw());
            context.bind_vertex_array(self.vertex_array.vertex_array_name);

            gl::enable_vertex_attrib_array(attrib_location);
            gl::vertex_attrib_pointer(
                attrib_location,
                layout.elements as i32,
                GlType::Float,
                False,
                (layout.stride * mem::size_of::<f32>()) as i32, // TODO: Correctly handle non-f32
                layout.offset * mem::size_of::<f32>());         // attrib data types.
        }

        self
    }

    /// Maps a vertex attribute to a variable name in the shader program.
    ///
    /// `map_attrib_name()` will silently ignore a program that does not have an input variable
    /// named `program_attrib_name` or a vertex buffer that does not have an attribute named
    /// `buffer_attrib_name`, so it is always safe to speculatively map vertex attributes
    /// even when the shader program may not use that attribute.
    ///
    /// # Panics
    ///
    /// - If the program has not been set using `program()`.
    pub fn map_attrib_name(
        &mut self,
        buffer_attrib_name: &str,
        program_attrib_name: &str
    ) -> &mut DrawBuilder<'a> {
        let program = self.program.expect("Cannot map attribs without a shader program");
        let attrib = match program.get_attrib(program_attrib_name) {
            Some(attrib) => attrib,
            None => return self,
        };
        let layout = match self.vertex_array.vertex_buffer.attribs.get(buffer_attrib_name) {
            Some(&attrib_data) => attrib_data,
            None => return self,
        };

        unsafe {
            let mut context = self.context.borrow_mut();
            let _guard = ::context::ContextGuard::new(context.raw());
            context.bind_vertex_array(self.vertex_array.vertex_array_name);

            gl::enable_vertex_attrib_array(attrib);
            gl::vertex_attrib_pointer(
                attrib,
                layout.elements as i32,
                GlType::Float,
                False,
                (layout.stride * mem::size_of::<f32>()) as i32,
                layout.offset * mem::size_of::<f32>());
        }

        self
    }

    /// Sets the value of a uniform variable in the shader program.
    ///
    /// `uniform()` will silently ignore uniform variables that do not exist in the shader program,
    /// so it is always safe to speculatively set uniform values even if the shader program may
    /// not use that uniform.
    ///
    /// # Panics
    ///
    /// - If the program has not been set using `program()`.
    pub fn uniform<T>(
        &mut self,
        name: &str,
        value: T
    ) -> &mut DrawBuilder<'a>
        where T: Into<UniformValue<'a>>
    {
        let value = value.into();

        let program =
            self.program.expect("Cannot set a uniform without a shader program");

        // TODO: This checking is bad? Or maybe not? I don't remember.
        let uniform_location = match program.get_uniform_location(name) {
            Some(location) => location,
            None => return self,
        };

        // Add uniform to the uniform map.
        self.uniforms.insert(uniform_location, value);

        self
    }

    pub fn draw(&mut self) {
        let mut context = self.context.borrow_mut();
        let _guard = ::context::ContextGuard::new(context.raw());

        context.polygon_mode(self.polygon_mode.unwrap_or_default());
        context.use_program(self.program.map(Program::inner));

        if let Some(face) = self.cull {
            context.enable_server_cull(true);
            context.cull_mode(face);
            context.winding_order(self.winding_order);
        } else {
            context.enable_server_cull(false);
        }

        if let Some(depth_test) = self.depth_test {
            context.enable_server_depth_test(true);
            context.depth_test(depth_test);
        } else {
            context.enable_server_depth_test(false);
        }

        let (source_factor, dest_factor) = self.blend;
        context.blend(source_factor, dest_factor);

        let mut active_texture = 0;
        // Apply uniforms.
        for (&location, uniform) in &self.uniforms {
            self.apply(uniform, location, &mut active_texture);
        }

        unsafe {
            // TODO: Do a better job tracking VAO and VBO state? I don't know how that would be
            // accomplished, but I don't honestly undertand VAOs so maybe I should figure that out
            // first.
            context.bind_vertex_array(self.vertex_array.vertex_array_name);

            if let Some(indices) = self.vertex_array.index_buffer.as_ref() {
                gl::draw_elements(
                    self.draw_mode,
                    indices.len as i32,
                    IndexType::UnsignedInt,
                    0);
            } else {
                gl::draw_arrays(
                    self.draw_mode,
                    0,
                    self.vertex_array.vertex_buffer.element_len as i32);
            }
        }
    }

    fn apply(&self, uniform: &UniformValue, location: UniformLocation, active_texture: &mut i32) {
        match *uniform {
            UniformValue::f32(value) => unsafe {
                gl::uniform_f32x1(location, value);
            },
            UniformValue::f32x2((x, y)) => unsafe {
                gl::uniform_f32x2(location, x, y);
            },
            UniformValue::f32x3((x, y, z)) => unsafe {
                gl::uniform_f32x3(location, x, y, z);
            },
            UniformValue::f32x4((x, y, z, w)) => unsafe {
                gl::uniform_f32x4(location, x, y, z, w);
            },
            UniformValue::i32(value) => unsafe {
                gl::uniform_i32x1(location, value);
            },
            UniformValue::u32(value) => unsafe {
                gl::uniform_u32x1(location, value);
            },
            UniformValue::Matrix(ref matrix) => match matrix.data.len() {
                16 => unsafe {
                    gl::uniform_matrix_f32x4v(
                        location,
                        1,
                        matrix.transpose.into(),
                        matrix.data.as_ptr())
                },
                9 => unsafe {
                    gl::uniform_matrix_f32x3v(
                        location,
                        1,
                        matrix.transpose.into(),
                        matrix.data.as_ptr())
                },
                _ => panic!("Unsupported matrix data length: {}", matrix.data.len()),
            },
            UniformValue::Texture(texture) => {
                unsafe {
                    texture::set_active_texture(*active_texture as u32);
                    gl::bind_texture(TextureBindTarget::Texture2d, texture.inner());
                    gl::uniform_i32x1(location, *active_texture);
                }

                *active_texture += 1;
            }
        }
    }
}

/// Represents a value for a uniform variable in a shader program.
#[derive(Debug)]
#[allow(bad_style)]
pub enum UniformValue<'a> {
    f32(f32),
    f32x2((f32, f32)),
    f32x3((f32, f32, f32)),
    f32x4((f32, f32, f32, f32)),
    i32(i32),
    u32(u32),
    Matrix(GlMatrix<'a>),
    Texture(&'a Texture2d),
}

impl<'a> From<f32> for UniformValue<'a> {
    fn from(value: f32) -> UniformValue<'a> {
        UniformValue::f32(value)
    }
}

impl<'a> From<(f32, f32)> for UniformValue<'a> {
    fn from(value: (f32, f32)) -> UniformValue<'a> {
        UniformValue::f32x2(value)
    }
}

impl<'a> From<(f32, f32, f32)> for UniformValue<'a> {
    fn from(value: (f32, f32, f32)) -> UniformValue<'a> {
        UniformValue::f32x3(value)
    }
}

impl<'a> From<(f32, f32, f32, f32)> for UniformValue<'a> {
    fn from(value: (f32, f32, f32, f32)) -> UniformValue<'a> {
        UniformValue::f32x4(value)
    }
}

impl<'a> From<[f32; 1]> for UniformValue<'a> {
    fn from(value: [f32; 1]) -> UniformValue<'a> {
        UniformValue::f32(value[0])
    }
}

impl<'a> From<[f32; 2]> for UniformValue<'a> {
    fn from(value: [f32; 2]) -> UniformValue<'a> {
        UniformValue::f32x2((value[0], value[1]))
    }
}

impl<'a> From<[f32; 3]> for UniformValue<'a> {
    fn from(value: [f32; 3]) -> UniformValue<'a> {
        UniformValue::f32x3((value[0], value[1], value[2]))
    }
}

impl<'a> From<[f32; 4]> for UniformValue<'a> {
    fn from(value: [f32; 4]) -> UniformValue<'a> {
        UniformValue::f32x4((value[0], value[1], value[2], value[3]))
    }
}

impl<'a> From<i32> for UniformValue<'a> {
    fn from(from: i32) -> UniformValue<'a> {
        UniformValue::i32(from)
    }
}

impl<'a> From<u32> for UniformValue<'a> {
    fn from(from: u32) -> UniformValue<'a> {
        UniformValue::u32(from)
    }
}

impl<'a> From<GlMatrix<'a>> for UniformValue<'a> {
    fn from(matrix: GlMatrix<'a>) -> UniformValue<'a> {
        UniformValue::Matrix(matrix)
    }
}

impl<'a> From<&'a Texture2d> for UniformValue<'a> {
    fn from(from: &'a Texture2d) -> UniformValue<'a> {
        UniformValue::Texture(from)
    }
}

#[derive(Debug, Clone)]
pub struct GlMatrix<'a> {
    pub data: &'a [f32],
    pub transpose: bool,
}
