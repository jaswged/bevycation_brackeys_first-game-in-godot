pub mod components;
mod systems;

use bevy::{prelude::*};
use crate::character_controller::systems::*;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update,
                (
                    update_grounded,
                    movement,
                    apply_movement_damping,
                    flip_player_based_on_movement,
                    activate_pass_through_one_way_platform_system,
                )
                    .chain(),
            );
    }
}


