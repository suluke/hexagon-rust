extern crate nalgebra_glm as glm;
use glm::Vec2;
use std::time::Duration;

pub struct Obstacle {
  its_distance: f32,
  its_height: f32,
}

impl Obstacle {
  pub fn new(the_height: f32) -> Obstacle {
    Obstacle {
      its_distance: 0.,
      its_height: the_height,
    }
  }
  pub fn get_height(&self) -> f32 {
    self.its_height
  }
  pub fn get_distance(&self) -> f32 {
    self.its_distance
  }
}

pub struct Slot {
  its_width: f32,
  its_obstacles: Vec<Obstacle>,
}

impl Slot {
  fn new() -> Slot {
    Slot {
      its_width: 1.0,
      its_obstacles: Vec::new(),
    }
  }
  pub fn get_width(&self) -> f32 {
    self.its_width
  }
  pub fn get_obstacles(&self) -> &Vec<Obstacle> {
    &self.its_obstacles
  }
  pub fn add_obstacle(&mut self, the_obstacle: Obstacle) -> () {
    self.its_obstacles.push(the_obstacle);
  }
}

#[derive(Clone)]
pub struct Color {
  pub its_r: f32,
  pub its_g: f32,
  pub its_b: f32,
  pub its_a: f32,
}

impl Color {
  pub fn rgba(the_r: f32, the_g: f32, the_b: f32, the_a: f32) -> Color {
    Color {
      its_r: the_r,
      its_g: the_g,
      its_b: the_b,
      its_a: the_a,
    }
  }
}

pub struct Style {
  its_cursor_color: Color,
  its_cursor_shadow_color: Color,
  its_inner_hexagon_color: Color,
  its_outer_hexagon_color: Color,
  its_obstacle_color: Color,
  its_slot_colors: Vec<Color>,
  its_rotation: f32,
  its_zoom: f32,
  its_eye: Vec2,
  its_look_at: Vec2,
  its_flash_time: Duration,
}

impl Style {
  fn new() -> Style {
    Style {
      its_cursor_color: Color::rgba(0., 0., 1., 1.),
      its_cursor_shadow_color: Color::rgba(0., 0., 0., 0.),
      its_inner_hexagon_color: Color::rgba(0., 0., 0., 1.),
      its_outer_hexagon_color: Color::rgba(1., 0., 0., 1.),
      its_obstacle_color: Color::rgba(0., 1., 0., 1.),
      its_slot_colors: vec![Color::rgba(1., 0., 0., 1.), Color::rgba(1., 1., 1., 1.)],
      its_rotation: 0.,
      its_zoom: 1.,
      its_eye: Vec2::new(0., 0.),
      its_look_at: Vec2::new(0., 0.),
      its_flash_time: Duration::from_millis(0),
    }
  }

  pub fn get_eye(&self) -> &Vec2 {
    &self.its_eye
  }
  pub fn get_look_at(&self) -> &Vec2 {
    &self.its_look_at
  }
  pub fn get_rotation(&self) -> f32 {
    self.its_rotation
  }
  pub fn set_zoom(&mut self, the_zoom: f32) -> () {
    self.its_zoom = the_zoom;
  }
  pub fn get_zoom(&self) -> f32 {
    self.its_zoom
  }
  pub fn get_slot_colors(&self) -> &Vec<Color> {
    &self.its_slot_colors
  }
  pub fn get_obstacle_color(&self) -> &Color {
    &self.its_obstacle_color
  }
  pub fn get_outer_hexagon_color(&self) -> &Color {
    &self.its_outer_hexagon_color
  }
  pub fn get_inner_hexagon_color(&self) -> &Color {
    &self.its_inner_hexagon_color
  }
  pub fn get_cursor_color(&self) -> &Color {
    &self.its_cursor_color
  }
  pub fn get_cursor_shadow_color(&self) -> &Color {
    &self.its_cursor_shadow_color
  }
  pub fn get_flash_time(&self) -> std::time::Duration {
    self.its_flash_time
  }
}

pub struct GameState {
  its_player_position: f32,
  its_player_speed: f32,
  its_obstacle_speed: f32,
  its_slots: [Slot; 6],
  its_style: Style,
  its_is_running: bool,
}

impl GameState {
  pub fn new() -> GameState {
    GameState {
      its_player_position: 1. / 12.,
      its_player_speed: 0.03,
      its_obstacle_speed: 0.005,
      its_slots: [
        Slot::new(),
        Slot::new(),
        Slot::new(),
        Slot::new(),
        Slot::new(),
        Slot::new(),
      ],
      its_style: Style::new(),
      its_is_running: true,
    }
  }
  pub fn get_position(&self) -> f32 {
    self.its_player_position
  }
  pub fn set_position(&mut self, the_position: f32) -> () {
    self.its_player_position = the_position;
  }
  pub fn get_player_speed(&self) -> f32 {
    self.its_player_speed
  }
  pub fn get_slots(&self) -> &[Slot; 6] {
    &self.its_slots
  }
  pub fn get_style(&self) -> &Style {
    &self.its_style
  }
  pub fn get_style_mut(&mut self) -> &mut Style {
    &mut self.its_style
  }
  pub fn get_slot_idx_at_position(&self, the_position: f32) -> usize {
    // we are on a slot if it's a) wider than 0 and b) the slot's right
    // border is the first that is greater than position
    let slots = self.get_slots();
    let slot_width_sum = self.get_slot_width_sum(); // in [0, 6], position in [0, 1)
    let mut s = 0; // the index of the slot we're on according to `position`
                   // we are on slot s if position in [left, right).
    let mut x = slots[0].get_width();
    while x <= the_position * slot_width_sum {
      x += slots[(s + 1) % slots.len()].get_width();
      s += 1;
    }
    assert!(
      s < slots.len(),
      "Target slot out of bounds (${s}/${this.slots.length})",
    );
    return s;
  }
  pub fn get_current_slot_idx(&self) -> usize {
    self.get_slot_idx_at_position(self.its_player_position)
  }

  pub fn get_slot_width_sum(&self) -> f32 {
    self
      .its_slots
      .iter()
      .fold(0., |the_acc, the_slot| the_acc + the_slot.get_width())
  }
  pub fn is_running(&self) -> bool {
    self.its_is_running
  }
}
