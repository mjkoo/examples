//! # GLArea Sample
//!
//! This sample demonstrates how to use GLAreas and OpenGL

extern crate gtk;
extern crate libc;

#[cfg(feature = "opengl")]
extern crate epoxy;

// Based off of the static-triangle example from gl-rs
#[cfg(feature = "opengl")]
mod example {
    use std::ffi;
    use std::mem;
    use std::ptr;

    use gtk;
    use gtk::traits::*;
    use gtk::signal::Inhibit;
    use gtk::{GLArea, Window};

    use epoxy;
    use epoxy::types::*;
    use epoxy::Gl;

    fn compile_shader(src: &str, ty: GLenum) -> GLuint {
        let shader;
        unsafe {
            shader = Gl.CreateShader(ty);
            // Attempt to compile the shader
            let csrc = ffi::CString::new(src).unwrap().as_ptr();
            Gl.ShaderSource(shader, 1, &csrc, ptr::null());
            Gl.CompileShader(shader);

            // Get the compile status
            let mut status = epoxy::FALSE as GLint;
            Gl.GetShaderiv(shader, epoxy::COMPILE_STATUS, &mut status);

            // Fail on error
            if status != (epoxy::TRUE as GLint) {
                let mut len = 0;
                Gl.GetShaderiv(shader, epoxy::INFO_LOG_LENGTH, &mut len);
                let mut buf = vec![0u8; len as usize - 1];
                Gl.GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
                panic!("Error compiling shader");
            }
        }
        shader
    }

    fn link_program(vs: GLuint, fs: GLuint) -> GLuint { unsafe {
        let program = Gl.CreateProgram();
        Gl.AttachShader(program, vs);
        Gl.AttachShader(program, fs);
        Gl.LinkProgram(program);

        // Get the link status
        let mut status = epoxy::FALSE as GLint;
        Gl.GetProgramiv(program, epoxy::LINK_STATUS, &mut status);

        // Fail on error
        if status != (epoxy::TRUE as GLint) {
            let mut len: GLint = 0;
            Gl.GetProgramiv(program, epoxy::INFO_LOG_LENGTH, &mut len);
            let mut buf = vec![0u8; len as usize - 1];
            Gl.GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("Error linking shader");
        }

        program
    }}

    pub fn main() {
        if gtk::init().is_err() {
            println!("Failed to initialize GTK.");
            return;
        }

        let window = Window::new(gtk::WindowType::Toplevel).unwrap();
        let glarea = GLArea::new().unwrap();

        window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        glarea.connect_render(|_, _| {
            // TODO: Most of this should be done on realize, need to figure out issues with the
            // context then though
            let vertices: [GLfloat; 6] = [0.0, 0.5, 0.5, -0.5, -0.5, -0.5];

            let vert_shader_src = r#"
                #version 140

                in vec2 position;

                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                }"#;

            let frag_shader_src = r#"
                #version 140

                out vec4 color;

                void main() {
                    color = vec4(1.0, 0.0, 0.0, 1.0);
                }"#;

            let vs = compile_shader(vert_shader_src, epoxy::VERTEX_SHADER);
            let fs = compile_shader(frag_shader_src, epoxy::FRAGMENT_SHADER);
            let program = link_program(vs, fs);

            let mut vao: GLuint = 0;
            let mut vbo: GLuint = 0;

            unsafe {
                Gl.GenVertexArrays(1, &mut vao);
                Gl.BindVertexArray(vao);

                Gl.GenBuffers(1, &mut vbo);
                Gl.BindBuffer(epoxy::ARRAY_BUFFER, vbo);
                Gl.BufferData(epoxy::ARRAY_BUFFER,
                              (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                              mem::transmute(&vertices[0]),
                              epoxy::STATIC_DRAW);

                Gl.UseProgram(program);
                Gl.BindFragDataLocation(program, 0, ffi::CString::new("color").unwrap().as_ptr());

                let pos_attr = Gl.GetAttribLocation(program, ffi::CString::new("position").unwrap().as_ptr());
                Gl.EnableVertexAttribArray(pos_attr as GLuint);
                Gl.VertexAttribPointer(pos_attr as GLuint, 2, epoxy::FLOAT,
                                       epoxy::FALSE as GLboolean, 0, ptr::null());

                Gl.ClearColor(0.3, 0.3, 0.3, 1.0);
                Gl.Clear(epoxy::COLOR_BUFFER_BIT);

                Gl.DrawArrays(epoxy::TRIANGLES, 0, 3);
            };

            Inhibit(false)
        });

        window.set_title("GLArea Example");
        window.set_default_size(400, 400);
        window.add(&glarea);

        window.show_all();
        gtk::main();
    }
}

#[cfg(feature = "opengl")]
fn main() {
    example::main()
}

#[cfg(not(feature = "opengl"))]
fn main() {
    println!("Did you forget to build with `--features opengl`?");
}