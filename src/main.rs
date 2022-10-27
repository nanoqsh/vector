mod vert;

use glow::{Context, HasContext, NativeFramebuffer, NativeProgram, NativeTexture};

fn main() {
    use glutin::{
        dpi::PhysicalSize,
        event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        ContextBuilder,
    };

    let (el, window, gl) = unsafe {
        let el = EventLoop::new();
        let window_builder = WindowBuilder::new()
            .with_title("Vector")
            .with_inner_size(PhysicalSize::new(500, 500));

        let window = ContextBuilder::new()
            .with_vsync(true)
            .build_windowed(window_builder, &el)
            .expect("build window")
            .make_current()
            .expect("creation of window context");

        let gl = Context::from_loader_function(|s| window.get_proc_address(s) as *const _);
        (el, window, gl)
    };

    unsafe {
        gl.enable(glow::MULTISAMPLE);
    }

    let main_program = {
        let (vertex_shader_src, fragment_shader_src) = (
            r#"
                layout (location = 0) in vec2 pos;
                uniform mat2 mat;
                uniform vec2 transform;
                void main() {
                    vec2 p = mat * (transform + pos);
                    gl_Position = vec4(p, 0.0, 1.0);
                }
            "#,
            r#"
                precision mediump float;
                out vec4 color;
                void main() {
                    color = vec4(0.8, 0.5, 0.7, 1.0);
                }
            "#,
        );

        let program = create_shader_program(
            &gl,
            [
                (glow::VERTEX_SHADER, vertex_shader_src),
                (glow::FRAGMENT_SHADER, fragment_shader_src),
            ],
        );

        unsafe {
            gl.use_program(Some(program));
            let loc = gl
                .get_uniform_location(program, "mat")
                .expect("uniform location");

            let value: [f32; 4] = [1., 0., 0., 1.];
            gl.uniform_matrix_2_f32_slice(Some(&loc), false, &value);

            let loc = gl
                .get_uniform_location(program, "transform")
                .expect("uniform location");

            let [x, y]: [f32; 2] = [0., 0.];
            gl.uniform_2_f32(Some(&loc), x, y);
        }

        program
    };

    let screen_uniform_sample = 0;
    let post_program = {
        let (vertex_shader_src, fragment_shader_src) = (
            r#"
                const vec2 verts[4] = vec2[4](
                    vec2(0.0, 0.0),
                    vec2(0.0, 1.0),
                    vec2(1.0, 0.0),
                    vec2(1.0, 1.0)
                );
                out vec2 uv;
                void main() {
                    uv = verts[gl_VertexID];
                    gl_Position = vec4(uv * 2.0 - 1.0, 0.0, 1.0);
                }
            "#,
            r#"
                precision mediump float;
                uniform sampler2D screen;
                in vec2 uv;
                out vec4 color;
                void main() {
                    color = texture(screen, uv);
                }
            "#,
        );

        let program = create_shader_program(
            &gl,
            [
                (glow::VERTEX_SHADER, vertex_shader_src),
                (glow::FRAGMENT_SHADER, fragment_shader_src),
            ],
        );

        unsafe {
            gl.use_program(Some(program));
            let loc = gl
                .get_uniform_location(program, "screen")
                .expect("uniform location");

            gl.uniform_1_i32(Some(&loc), screen_uniform_sample);
        }

        program
    };

    let verts = vert::make_ellipse(40., 40.);
    let vertex_array = unsafe {
        use std::{mem, slice};

        let vertex_array = gl.create_vertex_array().expect("create a vertex array");
        gl.bind_vertex_array(Some(vertex_array));

        let array_buffer = gl.create_buffer().expect("create buffer");
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(array_buffer));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            slice::from_raw_parts(
                verts.as_ptr().cast(),
                verts.len() * mem::size_of_val(&verts[0]),
            ),
            glow::STATIC_DRAW,
        );

        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(
            0,
            2,
            glow::FLOAT,
            false,
            mem::size_of_val(&verts[0]).try_into().expect("convert"),
            0,
        );

        gl.bind_vertex_array(None);
        vertex_array
    };

    let mut screen_size = (500, 500);
    let mut frame = Frame::new(&gl, screen_size);

    if cfg!(debug_assertions) {
        unsafe {
            let err = gl.get_error();
            if err != glow::NO_ERROR {
                let errors = gl.get_debug_message_log(100);
                for err in errors {
                    eprintln!("{err:?}");
                }
            }
        }
    }

    let post_vertex_array = unsafe { gl.create_vertex_array().expect("create vertex array") };

    unsafe {
        gl.viewport(0, 0, screen_size.0 as i32, screen_size.1 as i32);
        gl.clear_color(0.2, 0.15, 0.4, 1.);
    }

    el.run(move |event, _, flow| {
        *flow = ControlFlow::Wait;
        match event {
            Event::MainEventsCleared => {
                window.window().request_redraw();
            }
            Event::RedrawRequested(_) => unsafe {
                gl.bind_framebuffer(glow::FRAMEBUFFER, Some(frame.framebuffer));

                // Draw an object
                gl.clear(glow::COLOR_BUFFER_BIT);
                gl.use_program(Some(main_program));
                gl.bind_vertex_array(Some(vertex_array));
                gl.draw_arrays(glow::TRIANGLE_FAN, 0, verts.len() as i32);

                // Blit multisampled buffer
                gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(frame.framebuffer));
                gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(frame.intermediate));
                gl.blit_framebuffer(
                    0,
                    0,
                    screen_size.0 as i32,
                    screen_size.1 as i32,
                    0,
                    0,
                    screen_size.0 as i32,
                    screen_size.1 as i32,
                    glow::COLOR_BUFFER_BIT,
                    glow::NEAREST,
                );

                // Draw frame
                gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                gl.use_program(Some(post_program));
                gl.bind_texture(glow::TEXTURE_2D, Some(frame.screen));
                gl.active_texture(screen_uniform_sample as u32);
                gl.bind_vertex_array(Some(post_vertex_array));
                gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);

                window.swap_buffers().expect("swap buffers");
            },
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    window.resize(size);

                    screen_size = size.into();

                    unsafe {
                        gl.viewport(0, 0, screen_size.0 as i32, screen_size.1 as i32);
                    }

                    frame.delete(&gl);
                    frame = Frame::new(&gl, screen_size);

                    unsafe {
                        gl.use_program(Some(main_program));
                        let loc = gl
                            .get_uniform_location(main_program, "mat")
                            .expect("uniform location");

                        let w = 2. / screen_size.0 as f32;
                        let h = 2. / screen_size.1 as f32;
                        let value: [f32; 4] = [w, 0., 0., h];
                        gl.uniform_matrix_2_f32_slice(Some(&loc), false, &value);
                    }
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Q),
                            ..
                        },
                    ..
                } => {
                    *flow = ControlFlow::Exit;
                }
                WindowEvent::CloseRequested => *flow = ControlFlow::Exit,
                _ => {}
            },
            _ => {}
        }
    });
}

fn create_shader_program(gl: &Context, shaders: [(u32, &str); 2]) -> NativeProgram {
    unsafe {
        let program = gl.create_program().expect("create the shader program");

        let shaders = shaders.map(|(shader_type, shader_src)| {
            let shader = gl.create_shader(shader_type).expect("create a shader");
            gl.shader_source(shader, &format!("#version 330\n{}", shader_src));
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!("{}", gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shader
        });

        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("{}", gl.get_program_info_log(program));
        }

        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }

        program
    }
}

struct Frame {
    framebuffer: NativeFramebuffer,
    intermediate: NativeFramebuffer,
    color_buffer: NativeTexture,
    screen: NativeTexture,
}

impl Frame {
    fn new(gl: &Context, (width, height): (u32, u32)) -> Self {
        const SAMPLES: u8 = 8;

        unsafe {
            let framebuffer = gl.create_framebuffer().expect("create framebuffer");
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));

            let color_buffer = gl.create_texture().expect("create texture");
            gl.bind_texture(glow::TEXTURE_2D_MULTISAMPLE, Some(color_buffer));
            gl.tex_image_2d_multisample(
                glow::TEXTURE_2D_MULTISAMPLE,
                SAMPLES as i32,
                glow::RGB as i32,
                width as i32,
                height as i32,
                true,
            );

            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D_MULTISAMPLE,
                Some(color_buffer),
                0,
            );

            if cfg!(debug_assertions)
                && gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE
            {
                panic!("the framebuffer incomplete");
            }

            let intermediate = gl.create_framebuffer().expect("create framebuffer");
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(intermediate));

            let screen = gl.create_texture().expect("create texture");
            gl.bind_texture(glow::TEXTURE_2D, Some(screen));
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

            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );

            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );

            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(screen),
                0,
            );

            if cfg!(debug_assertions)
                && gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE
            {
                panic!("the framebuffer incomplete");
            }

            Self {
                framebuffer,
                intermediate,
                color_buffer,
                screen,
            }
        }
    }

    fn delete(&mut self, gl: &Context) {
        unsafe {
            gl.delete_framebuffer(self.framebuffer);
            gl.delete_framebuffer(self.intermediate);
            gl.delete_texture(self.color_buffer);
            gl.delete_texture(self.screen);
        }
    }
}
