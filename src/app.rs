use super::controls;
use super::model;
use super::renderer;
use glutin::window::Window;
use std::cell::RefCell;
use std::time::Duration;

pub trait TweenAPI {
  fn get_window(&self) -> &Window;
  fn get_renderer(&self) -> &dyn renderer::Renderer;
  fn get_game_state_mut(&mut self) -> &mut model::GameState;
}

pub trait Tween {
  fn run(&mut self, the_progress: f32, the_app: &mut dyn TweenAPI) -> ();
}

struct FPSTween {}
impl FPSTween {
  pub fn new() -> FPSTween {
    FPSTween {}
  }
}
impl Tween for FPSTween {
  fn run(&mut self, _the_progress: f32, the_app: &mut dyn TweenAPI) -> () {
    let a_title = format!(
      "FPS: {}",
      (1000. / the_app.get_renderer().get_frame_time()) as u32
    );
    the_app.get_window().set_title(&a_title);
  }
}

struct ZoomTween {}
impl ZoomTween {
  pub fn new() -> ZoomTween {
    ZoomTween {}
  }
}
impl Tween for ZoomTween {
  fn run(&mut self, the_progress: f32, the_api: &mut dyn TweenAPI) -> () {
    the_api
      .get_game_state_mut()
      .get_style_mut()
      .set_zoom(0.5 + (std::f32::consts::PI * the_progress).sin() * 0.5);
  }
}

struct TweenInfo {
  its_duration: Duration,
  its_progress: Duration,
  its_cooldown: Duration,
  its_repetitions: i32,
}
impl TweenInfo {
  pub fn new(the_duration: Duration, the_cooldown: Duration, the_repetitions: i32) -> TweenInfo {
    TweenInfo {
      its_duration: the_duration,
      its_progress: Duration::from_secs(0),
      its_cooldown: the_cooldown,
      its_repetitions: the_repetitions,
    }
  }
}
struct TweenEngine {
  its_tweens: Vec<(RefCell<TweenInfo>, RefCell<Box<dyn Tween>>)>,
}
impl TweenEngine {
  pub fn new() -> TweenEngine {
    TweenEngine {
      its_tweens: Vec::new(),
    }
  }
  pub fn register(
    &mut self,
    the_tween: Box<dyn Tween>,
    the_duration: Duration,
    the_cooldown: Duration,
    the_repetitions: i32,
  ) -> () {
    let a_state = TweenInfo::new(the_duration, the_cooldown, the_repetitions);
    self
      .its_tweens
      .push((RefCell::new(a_state), RefCell::new(the_tween)))
  }
  pub fn tick(&self, the_api: &mut dyn TweenAPI, the_delta: Duration) -> () {
    for a_tween in &self.its_tweens {
      let (a_state_cell, a_action_cell) = a_tween;
      let mut a_state = a_state_cell.borrow_mut();
      let mut a_action = a_action_cell.borrow_mut();
      if a_state.its_repetitions == 0 {
        return;
      }
      a_state.its_progress += the_delta;
      if a_state.its_progress <= a_state.its_duration + the_delta {
        let a_progress = (a_state.its_progress.as_micros() as f32
          / a_state.its_duration.as_micros() as f32)
          .min(1.);
        a_action.run(a_progress, the_api);
      }

      if a_state.its_progress >= a_state.its_duration + a_state.its_cooldown {
        a_state.its_progress = Duration::from_secs(0);
        if a_state.its_repetitions > 0 {
          a_state.its_repetitions -= 1;
        }
      }
    }
  }
  pub fn cleanup(&mut self) -> () {}
}

struct AppTweenAPI<'g, 'r, 'w> {
  its_game_state: &'g mut model::GameState,
  its_renderer: &'r dyn renderer::Renderer,
  its_window: &'w Window,
}
impl<'g, 'r, 'w> AppTweenAPI<'g, 'r, 'w> {
  pub fn new(
    the_game: &'g mut model::GameState,
    the_renderer: &'r dyn renderer::Renderer,
    the_window: &'w Window,
  ) -> AppTweenAPI<'g, 'r, 'w> {
    AppTweenAPI {
      its_game_state: the_game,
      its_renderer: the_renderer,
      its_window: the_window,
    }
  }
}
impl<'g, 'a, 'w> TweenAPI for AppTweenAPI<'g, 'a, 'w> {
  fn get_window(&self) -> &Window {
    self.its_window
  }
  fn get_renderer(&self) -> &dyn renderer::Renderer {
    self.its_renderer
  }
  fn get_game_state_mut(&mut self) -> &mut model::GameState {
    self.its_game_state
  }
}

pub struct App<Renderer: renderer::Renderer> {
  its_game: model::GameState,
  its_controls: controls::Controls,
  its_renderer: Renderer,
  its_tweens: TweenEngine,
}

impl<Renderer: renderer::Renderer> App<Renderer> {
  pub fn new(
    the_game: model::GameState,
    the_controls: controls::Controls,
    the_renderer: Renderer,
  ) -> App<Renderer> {
    let mut a_app = App {
      its_game: the_game,
      its_controls: the_controls,
      its_renderer: the_renderer,
      its_tweens: TweenEngine::new(),
    };
    a_app.its_tweens.register(
      Box::new(FPSTween::new()),
      Duration::from_secs(0),
      Duration::from_secs(1),
      -1,
    );
    a_app.its_tweens.register(
      Box::new(ZoomTween::new()),
      Duration::from_secs(2),
      Duration::from_secs(0),
      -1,
    );

    a_app
  }
  pub fn get_controls(&mut self) -> &mut controls::Controls {
    &mut self.its_controls
  }
  pub fn get_renderer(&self) -> &Renderer {
    &self.its_renderer
  }
  pub fn get_renderer_mut(&mut self) -> &mut Renderer {
    &mut self.its_renderer
  }

  pub fn tick(&mut self, the_window: &Window, the_delta: Duration) -> () {
    self.its_controls.tick(&mut self.its_game, the_delta);
    self.its_tweens.tick(
      &mut AppTweenAPI::new(&mut self.its_game, &self.its_renderer, the_window),
      the_delta,
    );
    self.its_tweens.cleanup();
    self.its_renderer.render(&self.its_game, the_delta);
  }
}
