use super::{despawn_screen, GameState};
use bevy::prelude::*;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(game_over_setup))
            .add_system_set(SystemSet::on_update(GameState::GameOver).with_system(start_playing))
            .add_system_set(
                SystemSet::on_exit(GameState::GameOver)
                    .with_system(despawn_screen::<GameOverScreen>),
            );
    }
}

fn start_playing(mut game_state: ResMut<State<GameState>>, keys: Res<Input<KeyCode>>) {
    if keys.just_released(KeyCode::Escape) {
        game_state.set(GameState::Menu).unwrap();
    }
}

#[derive(Component)]
struct GameOverScreen;

fn game_over_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("Orbitron.ttf");

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                margin: Rect::all(Val::Auto),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: Color::BLACK.into(),
            ..Default::default()
        })
        .insert(GameOverScreen)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                style: Style {
                    margin: Rect::all(Val::Px(50.0)),
                    ..Default::default()
                },
                text: Text::with_section(
                    "GAME OVER",
                    TextStyle {
                        font: font.clone(),
                        font_size: 80.0,
                        color: Color::WHITE,
                    },
                    Default::default(),
                ),
                ..Default::default()
            });

            parent.spawn_bundle(TextBundle {
                style: Style {
                    margin: Rect::all(Val::Px(50.0)),
                    ..Default::default()
                },
                text: Text::with_section(
                    "PRESS [ESC] TO RETURN TO MENU",
                    TextStyle {
                        font: font.clone(),
                        font_size: 40.0,
                        color: Color::WHITE,
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        });
}
