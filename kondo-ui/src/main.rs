mod game;

use bevy::prelude::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(game::game_plugin);

    app.run();
}
