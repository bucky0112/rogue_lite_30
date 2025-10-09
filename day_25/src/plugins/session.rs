use crate::resources::GameSession;
use crate::systems::game_session::{
    RequestLoadGameEvent, RequestSaveGameEvent, ResumeGameplayEvent, StartNewGameEvent,
    activate_gameplay_after_start, handle_main_menu_interactions, handle_pause_menu_interactions,
    process_load_game_requests, process_save_game_requests, resume_gameplay, spawn_main_menu,
    toggle_pause_menu_on_escape,
};
use bevy::prelude::*;

pub struct SessionPlugin;

impl Plugin for SessionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameSession>()
            .add_event::<StartNewGameEvent>()
            .add_event::<RequestLoadGameEvent>()
            .add_event::<RequestSaveGameEvent>()
            .add_event::<ResumeGameplayEvent>()
            .add_systems(Startup, spawn_main_menu)
            .add_systems(
                Update,
                (
                    handle_main_menu_interactions,
                    activate_gameplay_after_start.after(handle_main_menu_interactions),
                    handle_pause_menu_interactions,
                    process_save_game_requests.after(handle_pause_menu_interactions),
                    process_load_game_requests
                        .after(handle_main_menu_interactions)
                        .after(handle_pause_menu_interactions)
                        .after(process_save_game_requests),
                    toggle_pause_menu_on_escape,
                    resume_gameplay
                        .after(handle_pause_menu_interactions)
                        .after(toggle_pause_menu_on_escape)
                        .after(process_load_game_requests),
                ),
            );
    }
}
