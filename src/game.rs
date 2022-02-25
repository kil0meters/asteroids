use std::f32::consts::PI;

use super::GameState;
use bevy::app::Events;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::{collide, Collision};
use rand::Rng;

use crate::despawn_screen;

#[derive(Component)]
struct Player {
    velocity: Vec2,
}

fn player_movement(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut bullet_fire_event: EventWriter<BulletFireEvent>,
    mut bullet_fire_timer: ResMut<BulletFireTimer>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    let (mut player, mut transform) = query.single_mut();
    let dt = time.delta_seconds_f64() as f32;
    let rotation = &mut transform.rotation;

    let angle = quat_to_angle(rotation);

    if keys.just_pressed(KeyCode::J) {
        bullet_fire_event.send(BulletFireEvent);
        bullet_fire_timer.0.reset()
    } else if keys.pressed(KeyCode::J) {
        if bullet_fire_timer.0.tick(time.delta()).just_finished() {
            bullet_fire_event.send(BulletFireEvent);
        }
    }

    if keys.pressed(KeyCode::W) {
        player.velocity.x += angle.cos() * 500.0 * dt;
        player.velocity.y += angle.sin() * 500.0 * dt;
    }
    if keys.pressed(KeyCode::S) {
        player.velocity.x += angle.cos() * -500.0 * dt;
        player.velocity.y += angle.sin() * -500.0 * dt;
    }

    if keys.pressed(KeyCode::A) {
        *rotation = rotation.mul_quat(Quat::from_rotation_z(2.0 * dt));
    }
    if keys.pressed(KeyCode::D) {
        *rotation = rotation.mul_quat(Quat::from_rotation_z(-2.0 * dt));
    }

    let magnitude = (player.velocity.x.powf(2.0) + player.velocity.y.powf(2.0)).sqrt();
    // If the total velocity is greater than 500, we normalize the vector
    if magnitude > 500.0 {
        player.velocity.x *= 500.0 / magnitude;
        player.velocity.y *= 500.0 / magnitude;
    }
    // Otherwise, we simply apply force for friction
    else if magnitude >= 0.2 {
        player.velocity.x *= (magnitude - 1.0) / magnitude;
        player.velocity.y *= (magnitude - 1.0) / magnitude;
    }
    // Set velocity to zero
    else {
        player.velocity.x = 0.0;
        player.velocity.y = 0.0;
    }

    let translation = &mut transform.translation;

    translation.x = translation.x + player.velocity.x * dt;
    translation.y = translation.y + player.velocity.y * dt;

    wrap_position(translation);
}

struct PlayerDeathEvent;

fn player_death(
    mut scoreboard: ResMut<Scoreboard>,
    mut death_event: EventReader<PlayerDeathEvent>,
    mut game_state: ResMut<State<GameState>>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    let (mut player, mut transform) = query.single_mut();

    for _ in death_event.iter() {
        scoreboard.lives -= 1;

        if scoreboard.lives < 0 {
            game_state.set(GameState::GameOver).unwrap();
        } else {
            player.velocity = Vec2::default();
            transform.rotation = Quat::from_rotation_z(PI / 2.0);
            transform.translation.x = 0.0;
            transform.translation.y = 0.0;
        }
    }
}

#[derive(Component)]
struct Asteroid {
    speed: f32,
    size: i32,
}

struct AsteroidTimer(Timer);

fn spawn_asteroid(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut timer: ResMut<AsteroidTimer>,
) {
    let asteroid_texture = asset_server.load("asteroid1.png");

    if timer.0.tick(time.delta()).just_finished() {
        let mut rng = rand::thread_rng();

        let translation = if rng.gen_bool(0.5) {
            Vec3::new(1280.0, rng.gen_range(0.0..720.0), 0.0)
        } else {
            Vec3::new(rng.gen_range(0.0..1280.0), 720.0, 0.0)
        };

        commands
            .spawn_bundle(SpriteBundle {
                transform: Transform {
                    translation,
                    rotation: Quat::from_rotation_z(rand::thread_rng().gen_range(-180.0..180.0)),
                    scale: Vec3::new(3.0, 3.0, 0.0),
                },
                sprite: Sprite {
                    color: Color::WHITE,
                    ..Default::default()
                },
                texture: asteroid_texture,
                ..Default::default()
            })
            .insert(Asteroid {
                speed: 100.0,
                size: 3,
            })
            .insert(Collider::Asteroid);
    }
}

fn asteroid_movement(time: Res<Time>, mut query: Query<(&mut Asteroid, &mut Transform)>) {
    let dt = time.delta_seconds_f64() as f32;

    for (asteroid, mut transform) in query.iter_mut() {
        let angle = quat_to_angle(&transform.rotation);

        let translation = &mut transform.translation;
        translation.x += angle.cos() * asteroid.speed * dt;
        translation.y += angle.sin() * asteroid.speed * dt;

        wrap_position(translation);
    }
}

fn asteroid_collision(
    mut commands: Commands,
    mut player_death_event: EventWriter<PlayerDeathEvent>,
    mut asteroid_query: Query<(Entity, &mut Asteroid, &Transform)>,
    mut score: ResMut<Scoreboard>,
    asset_server: Res<AssetServer>,
    collider_query: Query<(Entity, &Collider, &Transform)>,
) {
    let asteroid_texture = asset_server.load("asteroid1.png");

    for (asteroid_entity, asteroid, asteroid_transform) in asteroid_query.iter_mut() {
        for (collider_entity, collider, transform) in collider_query.iter() {
            let collision = collide(
                asteroid_transform.translation,
                asteroid_transform.scale.truncate() * 32.0,
                transform.translation,
                transform.scale.truncate(),
            );

            if let Some(_) = collision {
                match *collider {
                    Collider::Asteroid => {}
                    Collider::Bullet => {
                        commands.entity(collider_entity).despawn_recursive();
                        commands.entity(asteroid_entity).despawn_recursive();

                        if asteroid.size > 1 {
                            let angle = quat_to_angle(&asteroid_transform.rotation);
                            let new_size = asteroid.size - 1;

                            score.points += 100;

                            commands
                                .spawn_bundle(SpriteBundle {
                                    transform: Transform {
                                        translation: asteroid_transform.translation,
                                        rotation: Quat::from_rotation_z(angle + (PI / 4.0)),
                                        scale: Vec3::new(
                                            new_size as f32,
                                            new_size as f32,
                                            new_size as f32,
                                        ),
                                    },
                                    sprite: Sprite {
                                        color: Color::WHITE,
                                        ..Default::default()
                                    },
                                    texture: asteroid_texture.clone(),
                                    ..Default::default()
                                })
                                .insert(Asteroid {
                                    speed: asteroid.speed * 2.0,
                                    size: new_size,
                                })
                                .insert(Collider::Asteroid);

                            commands
                                .spawn_bundle(SpriteBundle {
                                    transform: Transform {
                                        translation: asteroid_transform.translation,
                                        rotation: Quat::from_rotation_z(angle - (PI / 4.0)),
                                        scale: Vec3::new(
                                            new_size as f32,
                                            new_size as f32,
                                            new_size as f32,
                                        ),
                                    },
                                    sprite: Sprite {
                                        color: Color::WHITE,
                                        ..Default::default()
                                    },
                                    texture: asteroid_texture.clone(),
                                    ..Default::default()
                                })
                                .insert(Asteroid {
                                    speed: asteroid.speed * 2.0,
                                    size: new_size,
                                })
                                .insert(Collider::Asteroid);
                        }
                    }
                    Collider::Player => {
                        player_death_event.send(PlayerDeathEvent);
                    }
                }
            }
        }
    }
}

#[derive(Component)]
struct Bullet {
    speed: f32,
}

struct BulletFireTimer(Timer);
struct BulletFireEvent;

fn bullet_movement(time: Res<Time>, mut query: Query<(&mut Bullet, &mut Transform)>) {
    let dt = time.delta_seconds_f64() as f32;

    for (bullet, mut transform) in query.iter_mut() {
        let angle = quat_to_angle(&transform.rotation);
        let translation = &mut transform.translation;

        translation.x += bullet.speed * angle.cos() * dt;
        translation.y += bullet.speed * angle.sin() * dt;

        wrap_position(translation);
    }
}

fn bullet_fire(
    mut commands: Commands,
    mut bullet_fire_event: EventReader<BulletFireEvent>,
    player_query: Query<(&mut Player, &mut Transform)>,
) {
    let (_, transform) = player_query.single();

    for _ in bullet_fire_event.iter() {
        let angle = quat_to_angle(&transform.rotation);

        commands
            .spawn_bundle(SpriteBundle {
                transform: Transform {
                    translation: transform.translation,
                    rotation: Quat::from_rotation_z(angle),
                    scale: Vec3::new(12.0, 12.0, 0.0),
                },
                sprite: Sprite {
                    color: Color::WHITE,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Bullet { speed: 1000.0 })
            .insert(Collider::Bullet);
    }
}

#[derive(Component)]
struct Scoreboard {
    lives: i32,
    points: i32,
}

fn scoreboard_update(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[0].value = format!("SCORE: {}\n", scoreboard.points);
    text.sections[1].value = format!("LIVES: {}", scoreboard.lives);
}

#[derive(Component)]
enum Collider {
    Asteroid,
    Bullet,
    Player,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerDeathEvent>()
            .add_event::<BulletFireEvent>()
            .insert_resource(Scoreboard {
                points: 0,
                lives: 3,
            })
            .insert_resource(AsteroidTimer(Timer::from_seconds(2.0, true)))
            .insert_resource(BulletFireTimer(Timer::from_seconds(0.3, true)))
            .add_system_set(SystemSet::on_enter(GameState::Game).with_system(game_setup))
            .add_system_set(
                SystemSet::on_update(GameState::Game)
                    .with_system(asteroid_movement)
                    .with_system(asteroid_collision)
                    .with_system(bullet_fire)
                    .with_system(bullet_movement)
                    .with_system(player_movement)
                    .with_system(player_death)
                    .with_system(spawn_asteroid)
                    .with_system(scoreboard_update),
            )
            .add_system_set(SystemSet::on_exit(GameState::Game).with_system(stop_game));
    }
}

fn game_setup(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    asset_server: Res<AssetServer>,
) {
    let player_texture = asset_server.load("player.png");
    let font = asset_server.load("Orbitron.ttf");

    scoreboard.points = 0;
    scoreboard.lives = 3;

    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "SCORE: ".into(),
                    style: TextStyle {
                        font: font.clone(),
                        font_size: 40.0,
                        color: Color::WHITE,
                    },
                },
                TextSection {
                    value: "LIVES: ".into(),
                    style: TextStyle {
                        font: font.clone(),
                        font_size: 40.0,
                        color: Color::WHITE,
                    },
                },
            ],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(12.0),
                right: Val::Px(12.0),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });

    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                rotation: Quat::from_rotation_z(PI / 2.0),
                scale: Vec3::new(1.0, 1.0, 0.0),
            },
            sprite: Sprite {
                color: Color::WHITE,
                ..Default::default()
            },
            texture: player_texture,
            ..Default::default()
        })
        .insert(Player {
            velocity: Vec2::new(0.0, 0.0),
        })
        .insert(Collider::Player);
}

fn stop_game(
    mut commands: Commands,
    asteroid_query: Query<(Entity, &Asteroid)>,
    player_query: Query<(Entity, &Player)>,
    text_query: Query<(Entity, &Text)>,
    bullet_query: Query<(Entity, &Bullet)>,
) {
    let (player, _) = player_query.single();
    commands.entity(player).despawn_recursive();

    for (asteroid, _) in asteroid_query.iter() {
        commands.entity(asteroid).despawn_recursive();
    }

    for (text, _) in text_query.iter() {
        commands.entity(text).despawn_recursive();
    }

    for (bullet, _) in bullet_query.iter() {
        commands.entity(bullet).despawn_recursive();
    }
}

// Wraps a position to the other side of the screen if the position is over the edge.
#[inline(always)]
fn wrap_position(translation: &mut Vec3) {
    if translation.x < -640.0 {
        translation.x = 640.0;
    } else if translation.x > 640.0 {
        translation.x = -640.0;
    }

    if translation.y < -360.0 {
        translation.y = 360.0;
    } else if translation.y > 360.0 {
        translation.y = -360.0;
    }
}

// Converts a quaternion to an angle about the z axis
fn quat_to_angle(rotation: &Quat) -> f32 {
    let (v, angle) = rotation.to_axis_angle();
    v.z * angle
}
