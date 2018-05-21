#![allow(non_upper_case_globals)]
extern crate glfw;
use self::glfw::{Context, Key, Action};

extern crate gl;
use self::gl::types::*;

use std::sync::mpsc::Receiver;
use std::ffi::CString;
use std::ptr;
use std::str;
use std::mem;
use std::os::raw::c_void;

// settings
const SCR_WIDTH: u32 = 1024;
const SCR_HEIGHT: u32 = 768;

const vertexShaderSource: &str = r#"
    #version 330 core
    in vec2 aPos;
    void main() {
        gl_Position = vec4(aPos, 0.0, 1.0);
    }
"#;

const fragmentShaderSource: &str = r#"
    #version 330 core
    out vec4 FragColor;
    uniform int windowSize;
    uniform int maxIterations;
    uniform float stepX;
    uniform float stepY;
    uniform float zoom;

    vec3 hsv2rgb(vec3 c) {
        vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
        vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
        return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
    }

    void main() {
        vec2 s = (gl_FragCoord.xy / windowSize * 4.0 - 2.0).xy;

        vec2 c = vec2((s.x / zoom) + stepX, (s.y / zoom) + stepY);
        vec2 z = c;

        int i;
        for(i = 0; i < maxIterations; i++) {
            z = vec2((z.x * z.x) - (z.y * z.y), 2.0 * z.x * z.y) + c;

            if (length(z) > 2.0) {
                break;
            }
        }

        if (length(z) <= 2.0) {
            FragColor = vec4(0.0, 0.0, 0.0, 1.0);
        } else {
            float val = float(i) / float(maxIterations);
            // FragColor = vec4(1.0, 1.0, 1.0, 1.0); // white
            // FragColor = vec4(val, val, val, 1.0); // black and white
            FragColor = vec4(hsv2rgb(vec3(val, 1.0, 1.0)), 1.0); // trippy
        }
    }
"#;

#[allow(non_snake_case)]
fn main() {
    // glfw: initialize and configure
    // ------------------------------
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    // glfw window creation
    // --------------------
    let (mut window, events) = glfw.create_window(SCR_WIDTH, SCR_HEIGHT, "LearnOpenGL", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_cursor_pos_polling(true);

    // gl: load all OpenGL function pointers
    // ---------------------------------------
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (shaderProgram, VAO) = unsafe {
        // build and compile our shader program
        // -------------------------------------
        // vertex shader
        let vertexShader = gl::CreateShader(gl::VERTEX_SHADER);
        let c_str_vert = CString::new(vertexShaderSource.as_bytes()).unwrap();
        gl::ShaderSource(vertexShader, 1, &c_str_vert.as_ptr(), ptr::null());
        gl::CompileShader(vertexShader);

        // check for shader complile errors
        let mut success = gl::FALSE as GLint;
        let mut infoLog = Vec::with_capacity(512);
        infoLog.set_len(512 - 1); // subtract 1 to skip the trailing null character
        gl::GetShaderiv(vertexShader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(vertexShader, 512, ptr::null_mut(), infoLog.as_mut_ptr() as *mut GLchar);
            println!("ERROR::SHADER::VERTEX::COMPILATION_FAILED\n{}", str::from_utf8(&infoLog).unwrap());
        }

        // fragment shader
        let fragmentShader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_str_frag = CString::new(fragmentShaderSource.as_bytes()).unwrap();
        gl::ShaderSource(fragmentShader, 1, &c_str_frag.as_ptr(), ptr::null());
        gl::CompileShader(fragmentShader);
        // check for shader compile errors
        gl::GetShaderiv(fragmentShader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(fragmentShader, 512, ptr::null_mut(), infoLog.as_mut_ptr() as *mut GLchar);
            println!("ERROR::SHADER::FRAGMENT::COMPILATION_FAILED\n{}", str::from_utf8(&infoLog).unwrap());
        }

        // link shader
        let shaderProgram = gl::CreateProgram();
        gl::AttachShader(shaderProgram, vertexShader);
        gl::AttachShader(shaderProgram, fragmentShader);
        gl::LinkProgram(shaderProgram);
        // check for linking errors
        gl::GetProgramiv(shaderProgram, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetProgramInfoLog(shaderProgram, 512, ptr::null_mut(), infoLog.as_mut_ptr() as *mut GLchar);
            println!("ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}", str::from_utf8(&infoLog).unwrap());
        }
        // cleanup
        gl::DeleteShader(vertexShader);
        gl::DeleteShader(fragmentShader);


        // TODO: Zoom and Pan

        // Zoom is just adding a multiplier to your x and y coordinates

        // Panning is a percentage of the zoom multiplier

        // set up vertex data (and buffer(s)) and configre vertex attributes
        // -----------------------------------------------------------------
        // HINT: type annotation is crucial since default for float literals is f64
        let vertices: [f32; 12] = [
            -1.0,  1.0, 0.0,
             1.0,  1.0, 0.0,
             1.0, -1.0, 0.0,
             -1.0, -1.0, 0.0,
        ];

        let indices = [
            0, 1, 3,
            1, 2, 3
        ];

        let (mut VBO, mut VAO, mut EBO) = (0, 0, 0);
        gl::GenVertexArrays(1, &mut VAO);
        gl::GenBuffers(1, &mut VBO);
        gl::GenBuffers(1, &mut EBO);
        // bind the Vertex Array Object first, then bind and set vertex buffer(s), and then configure vertex attribute(s).
        gl::BindVertexArray(VAO);

        gl::BindBuffer(gl::ARRAY_BUFFER, VBO);
        gl::BufferData(gl::ARRAY_BUFFER,
                       (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       &vertices[0] as *const f32 as *const c_void,
                       gl::STATIC_DRAW);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, EBO);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                       (indices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       &indices[0] as *const i32 as *const c_void,
                       gl::STATIC_DRAW);

        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * mem::size_of::<GLfloat>() as GLsizei, ptr::null());
        gl::EnableVertexAttribArray(0);

        // not that this is allowed, the call to gl::VertexAttribPointer registered VBO as the vertex attribute's bound vertex buffer object so afterwards we can safely unbind
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);

        // You can unbind the VAO afterwards so other VAO calls won't accidentally modify this VOA, but this rarely happens. Modifying other
        // VAOs requires a call to glBindVertexArray anyways so we generally don't unbind VAOs (nor VBOs) when it's not directly necessary.
        gl::BindVertexArray(0);

        // draw in wireframe polygons.
        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

        (shaderProgram, VAO)
    };

    let mut x = 0.75;
    let mut y = 0.0;
    let mut zoom = 1.0;

    let zoom_max: f64 = 1000.0;
    let mut keypress : bool = true;
    let mut first : bool = false;
    let mut second : bool = false;

    // render loop
    // -----------
    while !window.should_close() {
        glfw.poll_events();
        // events
        // -----

        process_events(&mut window, &events, &mut keypress, &mut x, &mut y, &mut zoom);

        // render
        // ------
        if zoom > zoom_max {
          zoom = zoom_max;
        }

        if keypress || first || second {
            keypress = false;
            first = false;
            second = false;
            unsafe {
                gl::ClearColor(0.2, 0.3, 0.3, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);

                let windowSize = CString::new("windowSize").unwrap();
                let stepX = CString::new("stepX").unwrap();
                let stepY = CString::new("stepY").unwrap();
                let maxIterations = CString::new("maxIterations").unwrap();
                let zoomStr = CString::new("zoom").unwrap();
                let windowSizeLocation = gl::GetUniformLocation(shaderProgram, windowSize.as_ptr());
                let stepXLocation = gl::GetUniformLocation(shaderProgram, stepX.as_ptr());
                let stepYLocation = gl::GetUniformLocation(shaderProgram, stepY.as_ptr());
                let maxIterationsLocation = gl::GetUniformLocation(shaderProgram, maxIterations.as_ptr());
                let zoomLocation = gl::GetUniformLocation(shaderProgram, zoomStr.as_ptr());

                let (w, h): (i32, i32) = window.get_framebuffer_size();
                let smaller: i32;
                if w >= h {
                    smaller = h;
                } else {
                    smaller= w;
                }

                let max_itrs: i32 = (100 + ((1.0f64 / zoom).log10().abs() as u32 * 8)) as i32;

                gl::Uniform1i(windowSizeLocation, smaller);
                gl::Uniform1f(stepXLocation, x as f32);
                gl::Uniform1f(stepYLocation, y as f32);
                gl::Uniform1f(zoomLocation, zoom as f32);
                gl::Uniform1i(maxIterationsLocation, max_itrs);

                println!("-x {} -y {} -z {} m {}", x, y, zoom, max_itrs);

                // draw out first triangle
                gl::UseProgram(shaderProgram);
                gl::BindVertexArray(VAO); // seeing as we only have a single VAO there's no need to bind it every time, but we'll do so to keep things a bit more organized.
                // gl::DrawArrays(gl::TRIANGLES, 0, 3);
                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
                window.swap_buffers();
            }
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
    }
}

// NOTE: not the same version as in common.rs!
fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>, keypress: &mut bool, x: &mut f64, y: &mut f64, zoom: &mut f64) -> () {
    let step:  f64 = 0.05 * (1.0 / *zoom);

    for (time, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.
                unsafe {
                    gl::Viewport(0, 0, width, height);
                }
            }
            glfw::WindowEvent::CursorPos(xpos, ypos) => window.set_title(&format!("Time: {:?}, Cursor position: ({:?}, {:?})", time, xpos, ypos)),
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
            glfw::WindowEvent::Key(Key::W, _, Action::Press, _) => {
                *keypress = true;
                *y -= step;
            },
            glfw::WindowEvent::Key(Key::S, _, Action::Press, _) => {
                *keypress = true;
                *y += step;
            },
            glfw::WindowEvent::Key(Key::D, _, Action::Press, _) => {
                *keypress = true;
                *x -= step;
            },
            glfw::WindowEvent::Key(Key::A, _, Action::Press, _) => {
                *keypress = true;
                *x += step;
            },
            glfw::WindowEvent::Key(Key::E, _, Action::Press, _) => {
                *keypress = true;
                *zoom *= 1.2;
            },
            glfw::WindowEvent::Key(Key::Q, _, Action::Press, _) => {
                *keypress = true;
                *zoom *= 0.8;
            },
            _ => {}
        }
    }
}
