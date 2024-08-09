use std::mem;
use glow::HasContext;

pub struct GLTextureRGB {
    gl: glow::Context,
    program: glow::Program,
    texture: glow::Texture,
    vbo: glow::Buffer,
    vao: glow::VertexArray,
    width: u32,
    height: u32,
}

impl GLTextureRGB {
    pub fn new(gl: glow::Context, width: u32, height: u32) -> Self {
        unsafe {
            // Compile and link shaders
            let program = gl.create_program().expect("Cannot create program");
            let vertex_shader = gl.create_shader(glow::VERTEX_SHADER).expect("Cannot create vertex shader");
            let fragment_shader = gl.create_shader(glow::FRAGMENT_SHADER).expect("Cannot create fragment shader");

            gl.shader_source(vertex_shader, include_str!("shaders/rgb/vertex.glsl"));
            gl.shader_source(fragment_shader, include_str!("shaders/rgb/fragment.glsl"));
            gl.compile_shader(vertex_shader);
            if !gl.get_shader_compile_status(vertex_shader) {
                panic!("Vertex shader compilation failed: {}", gl.get_shader_info_log(vertex_shader));
            }
            gl.compile_shader(fragment_shader);
            if !gl.get_shader_compile_status(fragment_shader) {
                panic!("Fragment shader compilation failed: {}", gl.get_shader_info_log(fragment_shader));
            }

            gl.attach_shader(program, vertex_shader);
            gl.attach_shader(program, fragment_shader);
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!("Program linking failed: {}", gl.get_program_info_log(program));
            }

            gl.detach_shader(program, vertex_shader);
            gl.delete_shader(vertex_shader);
            gl.detach_shader(program, fragment_shader);
            gl.delete_shader(fragment_shader);

            // Create and configure the texture
            let texture = gl.create_texture().expect("Cannot create texture");
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as i32,
                width as i32,
                height as i32,
                0,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                None,
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);

            // Define the vertices of the full-screen quad
            let vertices: [f32; 24] = [
                // positions   // texCoords (flipped)
                -1.0,  1.0,    0.0, 0.0,
                -1.0, -1.0,    0.0, 1.0,
                1.0, -1.0,    1.0, 1.0,
                1.0, -1.0,    1.0, 1.0,
                1.0,  1.0,    1.0, 0.0,
                -1.0,  1.0,    0.0, 0.0
            ];

            // Create and bind the VAO
            let vao = gl.create_vertex_array().expect("Cannot create vertex array");
            gl.bind_vertex_array(Some(vao));

            // Create and bind the VBO
            let vbo = gl.create_buffer().expect("Cannot create buffer");
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &vertices.align_to::<u8>().1, glow::STATIC_DRAW);

            // Configure the vertex attributes
            let stride = 4 * mem::size_of::<f32>() as i32;

            let position_location = gl.get_attrib_location(program, "position")
                .expect("Could not find attribute 'position' in shader");
            gl.vertex_attrib_pointer_f32(position_location as u32, 2, glow::FLOAT, false, stride, 0);
            gl.enable_vertex_attrib_array(position_location as u32);

            let tex_coords_location = gl.get_attrib_location(program, "texCoords")
                .expect("Could not find attribute 'texCoords' in shader");
            gl.vertex_attrib_pointer_f32(tex_coords_location as u32, 2, glow::FLOAT, false, stride, 2 * mem::size_of::<f32>() as i32);
            gl.enable_vertex_attrib_array(tex_coords_location as u32);

            Self {
                gl,
                program,
                texture,
                vbo,
                vao,
                width,
                height
            }
        }
    }

    pub fn update_with_frame(&self, frame_data: &[u8]) {
        // Ensure that the frame data has the expected length (width * height * 3 for RGB)
        // assert_eq!(frame_data.len(), (self.width * self.height * 3) as usize, "Frame data has an unexpected size!");

        unsafe {
            // Bind the texture
            self.gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));

            // Update the texture with the new frame data
            self.gl.tex_sub_image_2d(
                glow::TEXTURE_2D,
                0,
                0,
                0,
                self.width as i32,
                self.height as i32,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(frame_data),
            );
        }
    }

    pub fn render(&self) {
        unsafe {
            let gl = &self.gl;
            gl.use_program(Some(self.program));
            gl.clear_color(0.0, 0.0, 1.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
            // gl.viewport(0, 0, self.width as i32, self.height as i32);
            // Ensure the texture is bound to the correct texture unit
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));

            let texture_location = self.gl.get_uniform_location(self.program, "screenTexture");
            gl.uniform_1_i32(texture_location.as_ref(), 0); // 0 refers to texture unit 0
            
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo)); 
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(glow::TRIANGLES, 0, 6);
        }
    }
}

impl Drop for GLTextureRGB {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
            self.gl.delete_texture(self.texture);
            self.gl.delete_buffer(self.vbo);
            self.gl.delete_vertex_array(self.vao);
        }
    }
}

pub struct GLTextureYUV {
    gl: glow::Context,
    program: glow::Program,
    texture_y: glow::Texture,
    texture_u: glow::Texture,
    texture_v: glow::Texture,
    vbo: glow::Buffer,
    vao: glow::VertexArray,
    width: u32,
    height: u32,
}

impl GLTextureYUV {
    pub fn new(gl: glow::Context, width: u32, height: u32) -> Self {
        unsafe {
            // Compile and link shaders
            let program = gl.create_program().expect("Cannot create program");
            let vertex_shader = gl.create_shader(glow::VERTEX_SHADER).expect("Cannot create vertex shader");
            let fragment_shader = gl.create_shader(glow::FRAGMENT_SHADER).expect("Cannot create fragment shader");

            gl.shader_source(vertex_shader, include_str!("shaders/yuv/vertex.glsl"));
            gl.shader_source(fragment_shader, include_str!("shaders/yuv/fragment.glsl"));
            gl.compile_shader(vertex_shader);
            if !gl.get_shader_compile_status(vertex_shader) {
                panic!("Vertex shader compilation failed: {}", gl.get_shader_info_log(vertex_shader));
            }
            gl.compile_shader(fragment_shader);
            if !gl.get_shader_compile_status(fragment_shader) {
                panic!("Fragment shader compilation failed: {}", gl.get_shader_info_log(fragment_shader));
            }

            gl.attach_shader(program, vertex_shader);
            gl.attach_shader(program, fragment_shader);
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!("Program linking failed: {}", gl.get_program_info_log(program));
            }

            gl.detach_shader(program, vertex_shader);
            gl.delete_shader(vertex_shader);
            gl.detach_shader(program, fragment_shader);
            gl.delete_shader(fragment_shader);

            // Create and configure the textures
            let texture_y = gl.create_texture().expect("Cannot create texture");
            gl.bind_texture(glow::TEXTURE_2D, Some(texture_y));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::R16 as i32,
                width as i32,
                height as i32,
                0,
                glow::RED,
                glow::UNSIGNED_SHORT,
                None,
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);

            let texture_u = gl.create_texture().expect("Cannot create texture");
            gl.bind_texture(glow::TEXTURE_2D, Some(texture_u));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::R16 as i32,
                width as i32 / 2,
                height as i32 / 2,
                0,
                glow::RED,
                glow::UNSIGNED_SHORT,
                None,
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);

            let texture_v = gl.create_texture().expect("Cannot create texture");
            gl.bind_texture(glow::TEXTURE_2D, Some(texture_v));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::R16 as i32,
                width as i32 / 2,
                height as i32 / 2,
                0,
                glow::RED,
                glow::UNSIGNED_SHORT,
                None,
            );
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);

            // Define the vertices of the full-screen quad
            let vertices: [f32; 24] = [
                // positions   // texCoords (flipped)
                -1.0,  1.0,    0.0, 0.0,
                -1.0, -1.0,    0.0, 1.0,
                1.0, -1.0,    1.0, 1.0,
                1.0, -1.0,    1.0, 1.0,
                1.0,  1.0,    1.0, 0.0,
                -1.0,  1.0,    0.0, 0.0
            ];

            // Create and bind the VAO
            let vao = gl.create_vertex_array().expect("Cannot create vertex array");
            gl.bind_vertex_array(Some(vao));

            // Create and bind the VBO
            let vbo = gl.create_buffer().expect("Cannot create buffer");
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &vertices.align_to::<u8>().1, glow::STATIC_DRAW);

            // Configure the vertex attributes
            let stride = 4 * mem::size_of::<f32>() as i32;

            let position_location = gl.get_attrib_location(program, "position")
                .expect("Could not find attribute 'position' in shader");
            gl.vertex_attrib_pointer_f32(position_location as u32, 2, glow::FLOAT, false, stride, 0);
            gl.enable_vertex_attrib_array(position_location as u32);

            let tex_coords_location = gl.get_attrib_location(program, "texCoords")
                .expect("Could not find attribute 'texCoords' in shader");
            gl.vertex_attrib_pointer_f32(tex_coords_location as u32, 2, glow::FLOAT, false, stride, 2 * mem::size_of::<f32>() as i32);
            gl.enable_vertex_attrib_array(tex_coords_location as u32);

            Self {
                gl,
                program,
                texture_y,
                texture_u,
                texture_v,
                vbo,
                vao,
                width,
                height
            }
        }

    }

    pub fn read_video_frame(&self, video_frame: &ffmpeg_next::frame::Video) {
        let gl = &self.gl;
    
        let width = video_frame.width() as usize;
        let height = video_frame.height() as usize;
        let format = video_frame.format();
        
        let stride_y = video_frame.stride(0);
        let stride_u = video_frame.stride(1);
        let stride_v = video_frame.stride(2);

        let y_data = video_frame.data(0);
        let u_data = video_frame.data(1);
        let v_data = video_frame.data(2);

        unsafe {
            match format {
                ffmpeg_next::format::Pixel::YUV420P => {
                    // Update the Y texture
                    gl.bind_texture(glow::TEXTURE_2D, Some(self.texture_y));
                    gl.tex_sub_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        0,
                        0,
                        width as i32,
                        height as i32,
                        glow::RED,
                        glow::UNSIGNED_BYTE,
                        glow::PixelUnpackData::Slice(&y_data[..(stride_y * height)]),
                    );
    
                    // Update the U texture
                    gl.bind_texture(glow::TEXTURE_2D, Some(self.texture_u));
                    gl.tex_sub_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        0,
                        0,
                        (width / 2) as i32,
                        (height / 2) as i32,
                        glow::RED,
                        glow::UNSIGNED_BYTE,
                        glow::PixelUnpackData::Slice(&u_data[..(stride_u * (height / 2))]),
                    );
    
                    // Update the V texture
                    gl.bind_texture(glow::TEXTURE_2D, Some(self.texture_v));
                    gl.tex_sub_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        0,
                        0,
                        (width / 2) as i32,
                        (height / 2) as i32,
                        glow::RED,
                        glow::UNSIGNED_BYTE,
                        glow::PixelUnpackData::Slice(&v_data[..(stride_v * (height / 2))]),
                    );
                },
                ffmpeg_next::format::Pixel::YUV420P10LE => {
                    // Update the Y texture
                    gl.bind_texture(glow::TEXTURE_2D, Some(self.texture_y));
                    gl.tex_sub_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        0,
                        0,
                        width as i32,
                        height as i32,
                        glow::RED,
                        glow::UNSIGNED_SHORT,
                        glow::PixelUnpackData::Slice(&y_data[..(stride_y * height)]),
                    );
            
                    // Update the U texture
                    gl.bind_texture(glow::TEXTURE_2D, Some(self.texture_u));
                    gl.tex_sub_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        0,
                        0,
                        (width / 2) as i32,
                        (height / 2) as i32,
                        glow::RED,
                        glow::UNSIGNED_SHORT,
                        glow::PixelUnpackData::Slice(&u_data[..(stride_u * (height / 2))]),
                    );
            
                    // Update the V texture
                    gl.bind_texture(glow::TEXTURE_2D, Some(self.texture_v));
                    gl.tex_sub_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        0,
                        0,
                        (width / 2) as i32,
                        (height / 2) as i32,
                        glow::RED,
                        glow::UNSIGNED_SHORT,
                        glow::PixelUnpackData::Slice(&v_data[..(stride_v * (height / 2))]),
                    );
                },
                _ => {
                    panic!("Unsupported pixel format: {:?}", format);
                }
            }
        }
    }
    
    pub fn render(&self) {
        unsafe {
            let gl = &self.gl;
            gl.use_program(Some(self.program));
            gl.clear_color(0.0, 0.0, 1.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
            // gl.viewport(0, 0, self.width as i32, self.height as i32);
            // Ensure the texture is bound to the correct texture unit
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture_y));

            gl.active_texture(glow::TEXTURE1);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture_u));

            gl.active_texture(glow::TEXTURE2);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture_v));

            let texture_y_location = self.gl.get_uniform_location(self.program, "yTexture");
            gl.uniform_1_i32(texture_y_location.as_ref(), 0); // 0 refers to texture unit 0

            let texture_u_location = self.gl.get_uniform_location(self.program, "uTexture");
            gl.uniform_1_i32(texture_u_location.as_ref(), 1); // 1 refers to texture unit 1

            let texture_v_location = self.gl.get_uniform_location(self.program, "vTexture");
            gl.uniform_1_i32(texture_v_location.as_ref(), 2); // 2 refers to texture unit 2

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo)); 
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(glow::TRIANGLES, 0, 6);

        }
    }
}
