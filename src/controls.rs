use super::constants;
use super::model;

const LEFT_KEY: u32 = 105;
const RIGHT_KEY: u32 = 106;

pub struct Controls {
  /// All keys that are currently pressed
  its_keys: std::collections::BTreeSet<u32>,
  /**
   * Keys that have been pressed between the previous and
   * the present event loop iteration
   */
  its_new_keys: std::collections::BTreeSet<u32>,
}

impl Controls {
  pub fn new() -> Controls {
    Controls {
      its_keys: std::collections::BTreeSet::new(),
      its_new_keys: std::collections::BTreeSet::new(),
    }
  }
  pub fn key_pressed(&mut self, the_scancode: u32) -> () {
    self.its_keys.insert(the_scancode);
    self.its_new_keys.insert(the_scancode);
  }
  pub fn key_released(&mut self, the_scancode: u32) -> () {
    self.its_keys.remove(&the_scancode);
  }
  pub fn tick(&mut self, the_game: &mut model::GameState, the_delta: std::time::Duration) -> () {
    // Forward key information to key event listeners
    if self.its_new_keys.len() > 0 {
      // for key_listener in self.its_key_listeners {
      //   key_listener(newKeysDown);
      // }
      self.its_new_keys.clear();
    }
    // Apply controls on game state
    // TODO this feels like bad separation of concerns
    if !the_game.is_running() {
      return;
    }
    let effect = the_delta.as_millis() as f32 / constants::TARGET_TICK_TIME;
    let left = self.its_keys.contains(&LEFT_KEY);
    let right = self.its_keys.contains(&RIGHT_KEY);
    if (left || right) && !(left && right) {
      let a_move_dist = the_game.get_player_speed() * effect;
      let sign = if left { -1. } else { 1. };
      let mut newpos = the_game.get_position() + a_move_dist * sign;
      let wrapcorrection = if newpos >= 1. {
        -1.
      } else {
        if newpos < 0. {
          1.
        } else {
          0.
        }
      };
      newpos += wrapcorrection;
      // Check for sideways collisions
      let slots = the_game.get_slots();
      let slot_width_sum = the_game.get_slot_width_sum();
      let mut s = the_game.get_slot_idx_at_position(newpos); // the index of the slot we *should* move onto
      let target_slot = &slots[s];
      let cursor_tip = constants::CURSOR_Y + constants::CURSOR_H;
      for obstacle in target_slot.get_obstacles() {
        if obstacle.get_distance() <= cursor_tip
          && obstacle.get_distance() + obstacle.get_height() > cursor_tip
        {
          // collision - can't move here
          s = the_game.get_current_slot_idx();
          let mut pos_in_slot = slots
            .iter()
            .enumerate()
            .filter(|(idx, _)| idx < &s)
            .fold(0., |acc, (_, slot)| acc + slot.get_width());
          if right {
            pos_in_slot += slots[s].get_width() - 0.0001;
          }
          newpos = pos_in_slot / slot_width_sum;
          break;
        }
      }
      the_game.set_position(newpos);
    }
  }
}
