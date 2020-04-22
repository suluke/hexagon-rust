use super::constants;
use super::model;
use gl::types::*;
use glutin::{self, PossiblyCurrent};
use nalgebra_glm as glm;

pub trait Renderer {
    fn resize(&mut self, the_width: u32, the_height: u32) -> ();
    fn render(&mut self, the_game: &model::GameState, the_delta: std::time::Duration) -> ();
    /**
     * Get the (low-pass filtered) time between two frames in milliseconds
     */
    fn get_frame_time(&self) -> f32;
}

const FRAME_TIME_FILTER_STRENGTH: f32 = 20.;

const VS_SRC: &'static [u8] = b"
#version 100
precision mediump float;
attribute vec4 vertex;
uniform float rotation;
uniform float z_value;
uniform float zoom;
uniform mat4 proj;
float PI = 3.14159265359;
float SQRT2 = 1.41421356237;
void main() {
    // we want to rotate the the edge coordinates of the slots to be
    // placed equidistantly on a unit circle. Edge coordinates are in
    // the range [0, 1]. Therefore, 0 should be mapped to 0 degrees
    // rotation, 0.5 to 180 degrees etc. => the angle is x * 2 * PI
    float alpha = fract(vertex.x + rotation) * 2. * PI;
    // viewport is from -1 to 1 and an obstacle should become visible
    // as soon as its lower y coordinate is <= 1. Assuming aspect is
    // 1 for now, an obstacle coming from 45 degrees with distance
    // 1 will become visible at (1.0/1.0) => it should be sqrt(2)
    // away from the center
    float r = SQRT2;
    vec4 pos;
    // first, convert from \"normal\" xy coords to coords on circle
    pos.x = sin(alpha) * r;
    pos.y = cos(alpha) * r;
    // scale the point by distance to bottom
    pos *= vertex.y;
    // apply zoom
    pos.xy *= zoom;
    // prepare for projection
    pos.z = z_value;
    pos.w = 1.;
    pos = proj * pos;
    pos /= pos.w;
    pos.z = 0.;
    gl_Position = pos;
}
\0";

const FS_SRC: &'static [u8] = b"
#version 100
precision mediump float;
uniform vec3 color;
void main() {
    gl_FragColor = vec4(color, 1.0);
}
\0";

fn gl_check_error() -> () {
    unsafe {
        let a_error = gl::GetError();
        let a_error_msg = match a_error {
            gl::NO_ERROR => "No error",
            gl::INVALID_ENUM => "Invalid enum",
            gl::INVALID_VALUE => "Invalid value",
            gl::INVALID_OPERATION => "Invalid operation",
            gl::STACK_OVERFLOW => "Stack overflow",
            gl::STACK_UNDERFLOW => "Stack underflow",
            gl::OUT_OF_MEMORY => "Out of memory",
            _ => "Unknown error",
        };
        assert!(a_error == gl::NO_ERROR, a_error_msg);
    }
}

fn gl_get_uniform_location(the_program: GLuint, the_name: &str) -> Option<GLint> {
    unsafe {
        let a_name_c = std::ffi::CString::new(the_name).unwrap();
        let a_pos = gl::GetUniformLocation(the_program, a_name_c.as_ptr());
        if a_pos != -1 {
            Some(a_pos)
        } else {
            None
        }
    }
}
fn gl_get_attrib_location(the_program: GLuint, the_name: &str) -> Option<GLint> {
    unsafe {
        let a_name_c = std::ffi::CString::new(the_name).unwrap();
        let a_pos = gl::GetAttribLocation(the_program, a_name_c.as_ptr());
        if a_pos != -1 {
            Some(a_pos)
        } else {
            None
        }
    }
}

struct MatrixCache {
    its_view_mat: glm::Mat4,
    its_proj_mat: glm::Mat4,
    its_matrix: glm::Mat4,
    its_eye: glm::Vec2,
    its_lookat: glm::Vec2,
    its_aspect: f32,
}
impl MatrixCache {
    pub fn new(the_config: &model::Style, the_aspect: f32) -> MatrixCache {
        let mut a_mat_cache = MatrixCache {
            its_view_mat: glm::identity(),
            its_proj_mat: glm::identity(),
            its_matrix: glm::identity(),
            its_eye: the_config.get_eye().clone(),
            its_lookat: the_config.get_look_at().clone(),
            its_aspect: the_aspect,
        };
        a_mat_cache.compute_view();
        a_mat_cache.compute_proj();
        a_mat_cache.compute_matrix();
        a_mat_cache
    }
    fn compute_view(&mut self) -> () {
        let mut a_eye = glm::vec2_to_vec3(&self.its_eye);
        a_eye.z = 1.;
        let a_center = glm::vec2_to_vec3(&self.its_lookat);
        let a_up = glm::vec3(0., 1., 0.);
        self.its_view_mat = glm::look_at(&a_eye, &a_center, &a_up);
    }
    fn compute_proj(&mut self) -> () {
        self.its_proj_mat =
            glm::perspective(self.its_aspect, std::f32::consts::FRAC_PI_4, 0.1, 10.);
    }
    fn compute_matrix(&mut self) -> () {
        self.its_matrix = self.its_proj_mat * self.its_view_mat
    }
    pub fn get_matrix(&mut self, the_config: &model::Style, the_aspect: f32) -> &glm::Mat4 {
        let eye = the_config.get_eye();
        let lookat = the_config.get_look_at();
        let mut changed = false;
        // Check if the view matrix needs updating
        if eye[0] != self.its_eye[0]
            || eye[1] != self.its_eye[1]
            || lookat[0] != self.its_lookat[0]
            || lookat[1] != self.its_lookat[1]
        {
            changed = true;
            self.its_eye[0] = eye[0];
            self.its_eye[1] = eye[1];
            self.its_lookat[0] = lookat[0];
            self.its_lookat[1] = lookat[1];
            self.compute_view();
        }
        // Check if the projection matrix needs updating
        if the_aspect != self.its_aspect {
            changed = true;
            self.its_aspect = the_aspect;
            self.compute_proj();
        }
        // Any changes require a recomputation of the view-projection
        if changed {
            self.compute_matrix();
        }
        &self.its_matrix
    }
}

pub struct OGLRenderer {
    _its_program: u32,
    its_vertex_glbuf: u32,
    its_vertex_data: Vec<f32>,
    its_aspect: f32,
    its_matrix_cache: MatrixCache,
    its_zoom_loc: Option<GLint>,
    its_rotation_loc: Option<GLint>,
    its_z_loc: Option<GLint>,
    its_proj_loc: Option<GLint>,
    its_color_loc: GLint,
    its_vertex_loc: GLint,
    its_vertex_array_obj: GLuint,
    its_frame_time: f32,
}

impl OGLRenderer {
    pub fn new(
        the_game: &model::GameState,
        the_gl_context: &glutin::Context<PossiblyCurrent>,
        the_width: u32,
        the_height: u32,
    ) -> OGLRenderer {
        gl::load_with(|ptr| the_gl_context.get_proc_address(ptr) as *const _);
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            gl::Disable(gl::BLEND);
        }
        let a_buf_id = unsafe {
            let mut a_buf_id = std::mem::zeroed();
            gl::GenBuffers(1, &mut a_buf_id);
            a_buf_id
        };
        let a_program = OGLRenderer::create_program();
        let a_aspect = the_width as f32 / the_height as f32;
        let a_vao = unsafe {
            let mut a_vao = std::mem::zeroed();
            if gl::BindVertexArray::is_loaded() {
                gl::GenVertexArrays(1, &mut a_vao);
            }
            a_vao
        };
        let a_renderer = OGLRenderer {
            _its_program: a_program,
            its_vertex_glbuf: a_buf_id,
            its_vertex_data: Vec::new(),
            its_aspect: a_aspect,
            its_matrix_cache: MatrixCache::new(the_game.get_style(), a_aspect),
            its_zoom_loc: gl_get_uniform_location(a_program, "zoom"),
            its_rotation_loc: gl_get_uniform_location(a_program, "rotation"),
            its_z_loc: gl_get_uniform_location(a_program, "z_value"),
            its_proj_loc: gl_get_uniform_location(a_program, "proj"),
            its_color_loc: gl_get_uniform_location(a_program, "color").unwrap(),
            its_vertex_loc: gl_get_attrib_location(a_program, "vertex").unwrap(),
            its_vertex_array_obj: a_vao,
            its_frame_time: 0.,
        };
        a_renderer
    }

    fn create_program() -> u32 {
        unsafe {
            let vs = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(
                vs,
                1,
                [VS_SRC.as_ptr() as *const _].as_ptr(),
                std::ptr::null(),
            );
            gl::CompileShader(vs);
            let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(
                fs,
                1,
                [FS_SRC.as_ptr() as *const _].as_ptr(),
                std::ptr::null(),
            );
            gl::CompileShader(fs);
            let program = gl::CreateProgram();
            gl::AttachShader(program, vs);
            gl::AttachShader(program, fs);
            gl::LinkProgram(program);
            gl::UseProgram(program);
            gl_check_error();

            program
        }
    }

    fn get_projection_matrix(&mut self, the_config: &model::Style) -> &glm::Mat4 {
        self.its_matrix_cache
            .get_matrix(the_config, self.its_aspect)
    }

    fn update_vertex_buffer(&mut self, the_game: &model::GameState) -> () {
        self.its_vertex_data.clear();
        // create outer hexagon vertices
        let mut push_vertex = |x: f32, y: f32| {
            self.its_vertex_data.push(x);
            self.its_vertex_data.push(y);
        };
        push_vertex(0., 0.);
        for i in 0..the_game.get_slots().len() + 1 {
            push_vertex((i as f32 / 6.).fract(), constants::OUTER_HEXAGON_Y);
        }
        // create inner hexagon vertices
        push_vertex(0., 0.);
        for i in 0..the_game.get_slots().len() + 1 {
            push_vertex((i as f32 / 6.).fract(), constants::INNER_HEXAGON_Y);
        }
        // cursor coordinates
        let c_left = the_game.get_position() - constants::CURSOR_W / 2.;
        let c_right = the_game.get_position() + constants::CURSOR_W / 2.;
        let c_top = constants::CURSOR_Y + constants::CURSOR_H;
        // create cursorShadow vertices
        push_vertex(c_left, constants::CURSOR_Y);
        push_vertex(c_right, constants::CURSOR_Y);
        push_vertex(the_game.get_position(), c_top);
        // create cursor vertices
        push_vertex(c_left, constants::CURSOR_Y);
        push_vertex(c_right, constants::CURSOR_Y);
        push_vertex(the_game.get_position(), c_top);
        // create slot vertices
        let slot_width_sum = the_game.get_slot_width_sum();
        let mut x = 0.;
        let sl = 2.;
        for i in 0..the_game.get_slots().len() {
            push_vertex(x, 0.);
            push_vertex(x, sl);
            x += the_game.get_slots()[i].get_width() as f32 / slot_width_sum;
            push_vertex(x, 0.);
            push_vertex(x, sl);
        }
        // create obstacle vertices
        x = 0.;
        for s in 0..the_game.get_slots().len() {
            let slot = &the_game.get_slots()[s];
            let slot_width = slot.get_width() / slot_width_sum;
            for o in 0..slot.get_obstacles().len() {
                let obstacle = &slot.get_obstacles()[o];
                push_vertex(x, obstacle.get_distance().max(0.));
                push_vertex(x, obstacle.get_distance() + obstacle.get_height());
                push_vertex(x + slot_width, obstacle.get_distance().max(0.));
                push_vertex(
                    x + slot_width,
                    obstacle.get_distance() + obstacle.get_height(),
                );
            }
            x += slot_width;
        }
    }
}

impl Renderer for OGLRenderer {
    fn resize(&mut self, the_width: u32, the_height: u32) -> () {
        unsafe {
            self.its_aspect = the_width as f32 / the_height as f32;
            gl::Viewport(0, 0, the_width as GLsizei, the_height as GLsizei);
        }
    }
    fn render(&mut self, the_game: &model::GameState, the_delta: std::time::Duration) -> () {
        self.its_frame_time +=
            (the_delta.as_millis() as f32 - self.its_frame_time) / FRAME_TIME_FILTER_STRENGTH;

        let a_clear_color = model::Color::rgba(0., 0., 0., 1.);
        unsafe {
            let config = the_game.get_style();
            if config.get_flash_time().as_millis() > 0 {
                gl::ClearColor(1.0, 1.0, 1.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                return;
            }
            gl::ClearColor(
                a_clear_color.its_r,
                a_clear_color.its_g,
                a_clear_color.its_b,
                a_clear_color.its_a,
            );
            gl::Clear(gl::COLOR_BUFFER_BIT);

            if let Some(rotation_loc) = self.its_rotation_loc {
                gl::Uniform1f(rotation_loc, config.get_rotation());
            }
            // The longer dimension will see the full viewport - which is a 1x1 square.
            // Since by default we project to have x coordinates go from -1 to 1,
            // we only need to zoom if y is longer - i.e. aspect is less than zero
            //const aspect = gl.canvas.width / gl.canvas.height;
            let aspect_zoom = if self.its_aspect >= 1. {
                self.its_aspect
            } else {
                1.
            };
            let zoom = config.get_zoom() * aspect_zoom;
            if let Some(zoom_loc) = self.its_zoom_loc {
                gl::Uniform1f(zoom_loc, zoom);
            }
            if let Some(z_loc) = self.its_z_loc {
                assert!(z_loc != -1);
                gl::Uniform1f(z_loc, 0.);
            }
            if let Some(proj_loc) = self.its_proj_loc {
                let proj = self.get_projection_matrix(the_game.get_style());
                gl::UniformMatrix4fv(
                    proj_loc,
                    1 as gl::types::GLsizei,
                    gl::TRUE,
                    proj.as_ptr() as *const _,
                );
                gl_check_error();
            }

            // render slots
            self.update_vertex_buffer(the_game);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.its_vertex_glbuf);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.its_vertex_data.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                self.its_vertex_data.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            if gl::BindVertexArray::is_loaded() {
                gl::BindVertexArray(self.its_vertex_array_obj);
            }
            gl_check_error();
            gl::VertexAttribPointer(
                self.its_vertex_loc as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE,
                0,
                std::mem::zeroed(),
            );
            gl::EnableVertexAttribArray(self.its_vertex_loc as GLuint);
            gl_check_error();

            let a_color_loc = self.its_color_loc;
            // inner hex + outer hex + cursor + cursorShadow
            let num_hex_vertices = 8;
            let mut offset = 2 * num_hex_vertices + 3 + 3;
            let a_slot_colors = config.get_slot_colors();
            for i in 0..the_game.get_slots().len() {
                let a_slot_colr = if a_slot_colors.len() == 0 {
                    model::Color::rgba(1., 1., 1., 1.)
                } else {
                    a_slot_colors[i % a_slot_colors.len()].clone()
                };
                gl::Uniform3f(
                    a_color_loc,
                    a_slot_colr.its_r,
                    a_slot_colr.its_g,
                    a_slot_colr.its_b,
                );
                gl::DrawArrays(gl::TRIANGLE_STRIP, offset, 4);
                offset += 4;
            }
            gl_check_error();

            // render obstacles
            let obstacle_count = the_game
                .get_slots()
                .iter()
                .fold(0, |acc, slot| acc + slot.get_obstacles().len());
            let a_obst_colr = config.get_obstacle_color();
            gl::Uniform3f(
                a_color_loc,
                a_obst_colr.its_r,
                a_obst_colr.its_g,
                a_obst_colr.its_b,
            );
            for _ in 0..obstacle_count {
                gl::DrawArrays(gl::TRIANGLE_STRIP, offset, 4);
                offset += 4;
            }
            offset = 0;
            // render outer hexagon
            let a_oh_colr = config.get_outer_hexagon_color();
            gl::Uniform3f(
                a_color_loc,
                a_oh_colr.its_r,
                a_oh_colr.its_g,
                a_oh_colr.its_b,
            );
            gl::DrawArrays(gl::TRIANGLE_FAN, offset, num_hex_vertices);
            offset += num_hex_vertices;
            // render inner hexagon
            let a_ih_colr = config.get_inner_hexagon_color();
            gl::Uniform3f(
                a_color_loc,
                a_ih_colr.its_r,
                a_ih_colr.its_g,
                a_ih_colr.its_b,
            );
            gl::DrawArrays(gl::TRIANGLE_FAN, offset, num_hex_vertices);
            offset += num_hex_vertices;
            // render cursor shadow
            let a_shadow_color = config.get_cursor_shadow_color();
            if a_shadow_color.its_a != 0. {
                if let Some(z_loc) = self.its_z_loc {
                    gl::Uniform1f(z_loc, -0.01);
                    gl::Uniform3f(
                        a_color_loc,
                        a_shadow_color.its_r,
                        a_shadow_color.its_g,
                        a_shadow_color.its_b,
                    );
                    gl::DrawArrays(gl::TRIANGLES, offset, 3);
                    gl::Uniform1f(z_loc, 0.);
                }
            }
            offset += 3;
            // render cursor
            let a_cursor_colr = config.get_cursor_color();
            gl::Uniform3f(
                a_color_loc,
                a_cursor_colr.its_r,
                a_cursor_colr.its_g,
                a_cursor_colr.its_b,
            );
            gl::DrawArrays(gl::TRIANGLES, offset, 3);

            gl::Flush();
        }
    }

    /**
     * Get the (low-pass filtered) time between two frames in milliseconds
     */
    fn get_frame_time(&self) -> f32 {
        self.its_frame_time
    }
}
