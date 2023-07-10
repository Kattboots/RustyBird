use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use bevy_prototype_debug_lines::*;
use rand::prelude::*;

const TIME_STEP: f32 = 1.0 / 60.0;

const DRAGON_STARTPOSITION: Vec3 = Vec3::new(-350., 0., 0.);

const DRAGON_SIZE: Vec3 = Vec3::new(1.0, 1.0, 1.0);

const TERRAIN_SIZE: Vec3 = Vec3::new(1.0, 1.0, 1.0);

const BOUNDRY_SCALE: Vec3 = Vec3::new(1.0, 1.0, 1.0);

const BOUNDRY_DISTANCE: f32 = 200.;

const BOUNDY_SEGMENTS: usize = 16;

const DRAW_COLLIDER_BOXES: bool = false;

const HALF_SPRITE_DIMENSION: f32 = 32.0;

const SPRITE_DIMENSION: f32 = 64.0;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum RunState {
    Running,
    ResetTerrain,
}

#[derive(Resource)]
struct GameState {
    state: RunState,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Terrain;

#[derive(Component)]
struct LevelBoundry;
#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct Collider;

#[derive(Default)]
struct CollisionEvent;

#[derive(Resource)]
struct Scoreboard {
    score: usize,
    seconds_since_round_start: usize,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("dragon.png"),
            transform: Transform::from_xyz(
                DRAGON_STARTPOSITION.x,
                DRAGON_STARTPOSITION.y,
                DRAGON_STARTPOSITION.z,
            )
            .with_scale(DRAGON_SIZE),
            ..Default::default()
        })
        .insert(Player);

    let mut rng = thread_rng();

    for i in 0..8 {
        commands
            .spawn(SpriteBundle {
                texture: asset_server.load("terrain.png"),
                transform: Transform::from_xyz(
                    400. + (300. * i as f32),
                    10. * rng.gen_range(-10.0..10.0),
                    0.,
                )
                .with_scale(TERRAIN_SIZE),
                ..Default::default()
            })
            .insert(Terrain)
            .insert(Collider);
    }

    for i in 1..BOUNDY_SEGMENTS {
        // Roof
        commands
            .spawn(SpriteBundle {
                texture: asset_server.load("terrain.png"),
                transform: Transform::from_xyz(-450.0 + (64.0 * i as f32), BOUNDRY_DISTANCE, 0.)
                    .with_scale(BOUNDRY_SCALE),
                ..Default::default()
            })
            .insert(Terrain)
            .insert(LevelBoundry)
            .insert(Collider);

        // Floor
        commands
            .spawn(SpriteBundle {
                texture: asset_server.load("terrain.png"),
                transform: Transform::from_xyz(-450.0 + (64.0 * i as f32), -BOUNDRY_DISTANCE, 0.)
                    .with_scale(BOUNDRY_SCALE),
                ..Default::default()
            })
            .insert(Terrain)
            .insert(LevelBoundry)
            .insert(Collider);
    }

    // Text with multiple sections
    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    font: asset_server.load("Raleway-Bold.ttf"),
                    font_size: 32.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("Raleway-Bold.ttf"),
                font_size: 32.0,
                color: Color::GOLD,
            }),
        ])
        .with_text_alignment(TextAlignment::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                right: Val::Px(15.0),
                bottom: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        }),
        FpsText,
    ));

    // Text with multiple sections
    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font: asset_server.load("Raleway-Bold.ttf"),
                    font_size: 32.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("Raleway-Bold.ttf"),
                font_size: 32.0,
                color: Color::GOLD,
            }),
        ])
        .with_text_alignment(TextAlignment::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(10.0),
                left: Val::Px(15.0),
                ..Default::default()
            },
            ..Default::default()
        }),
        ScoreText,
    ));
}

fn simulated_gravity(time: Res<Time>, mut query: Query<(&mut Player, &mut Transform)>) {
    for (_player, mut transform) in query.iter_mut() {
        transform.translation.y -= 150. * time.delta_seconds();
    }
}

fn player_movement_system(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut sprite_position: Query<(&mut Player, &mut Transform)>,
) {
    for (_player, mut transform) in sprite_position.iter_mut() {
        if keys.pressed(KeyCode::Space) {
            transform.translation.y += 400. * time.delta_seconds();
        }
    }
}

fn move_terrain_forward(
    time: Res<Time>,
    mut query: Query<(&mut Terrain, &mut Transform), Without<LevelBoundry>>,
) {
    for (_terrain, mut transform) in query.iter_mut() {
        transform.translation.x -= 200. * time.delta_seconds();
    }
}

fn recycle_terrain(mut query: Query<(&mut Terrain, &mut Transform), Without<LevelBoundry>>) {
    for (_terrain, mut transform) in query.iter_mut() {
        if transform.translation.x < -1000. {
            let mut rng = thread_rng();
            transform.translation.x = 1000.;
            transform.translation.y = 10. * rng.gen_range(-10.0..10.0);
        }
    }
}

fn reset_terrain(
    mut query: Query<(&mut Terrain, &mut Transform), Without<LevelBoundry>>,
    mut game_state: ResMut<GameState>,
) {
    if game_state.state == RunState::ResetTerrain {
        println!("Resetting terrain");
        let mut i = 0;
        for (_terrain, mut transform) in query.iter_mut() {
            let mut rng = thread_rng();
            transform.translation.x = 1000. + (300. * i as f32);
            transform.translation.y = 10. * rng.gen_range(-10.0..10.0);
            i = i + 1;
        }

        game_state.state = RunState::Running;
    }
}

fn check_collisions(
    terrain_query: Query<(&Terrain, &mut Transform), With<Collider>>,
    mut player_query: Query<(&Player, &mut Transform), Without<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
    mut scoreboard: ResMut<Scoreboard>,
    time: Res<Time>,
) {
    let (_player, mut player_transform) = player_query.single_mut();
    let mut player_size = player_transform.scale.truncate();

    player_size.x *= SPRITE_DIMENSION;
    player_size.y *= SPRITE_DIMENSION;

    for (_terrain, collider_transform) in &terrain_query {
        let mut collider_size = collider_transform.scale.truncate();
        collider_size.x *= SPRITE_DIMENSION;
        collider_size.y *= SPRITE_DIMENSION;

        let collision = collide(
            player_transform.translation,
            player_size,
            collider_transform.translation,
            collider_size,
        );

        if let Some(collision) = collision {
            collision_events.send_default();

            match collision {
                Collision::Left => {}
                Collision::Right => {}
                Collision::Top => {}
                Collision::Bottom => {}
                Collision::Inside => {}
            }
        }
    }
}

fn text_update_system(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}

fn update_scoreboard(mut scoreboard: ResMut<Scoreboard>, time: Res<Time>) {
    let elapsed_seconds = time.elapsed_seconds() as usize;
    let score = (elapsed_seconds - scoreboard.seconds_since_round_start);
    scoreboard.score = score;
}

fn render_score(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text, With<ScoreText>>) {
    let mut score_text = query.single_mut();
    score_text.sections[1].value = scoreboard.score.to_string();
}

fn draw_debug_box(lines: &mut ResMut<DebugLines>, collider_transform: &Transform) {
    let display_duration = 0.0;

    let collider_translation = collider_transform.translation;
    let collider_scale = collider_transform.scale;

    if DRAW_COLLIDER_BOXES {
        let top_left = Vec3::new(
            collider_translation.x - (collider_scale.x * HALF_SPRITE_DIMENSION),
            collider_translation.y + (collider_scale.y * HALF_SPRITE_DIMENSION),
            0.0,
        );
        let bottom_left = Vec3::new(
            collider_translation.x - (collider_scale.x * HALF_SPRITE_DIMENSION),
            collider_translation.y - (collider_scale.y * HALF_SPRITE_DIMENSION),
            0.0,
        );
        let top_right = Vec3::new(
            collider_translation.x + (collider_scale.x * HALF_SPRITE_DIMENSION),
            collider_translation.y + (collider_scale.y * HALF_SPRITE_DIMENSION),
            0.0,
        );
        let bottom_right = Vec3::new(
            collider_translation.x + collider_scale.x + 32.0,
            collider_translation.y - collider_scale.y - 32.0,
            0.0,
        );

        lines.line(top_left, top_right, display_duration);
        lines.line(top_right, bottom_right, display_duration);
        lines.line(top_left, bottom_left, display_duration);
        lines.line(bottom_left, bottom_right, display_duration);
    }
}

fn draw_debug_lines(
    mut lines: ResMut<DebugLines>,
    player_collider: Query<&Transform, With<Player>>,
    terrain_colliders: Query<&Transform, With<Collider>>,
) {
    let player_transform = player_collider.single();

    draw_debug_box(&mut lines, &player_transform);

    for terrain_transform in terrain_colliders.iter() {
        let terrain_translation = terrain_transform;

        draw_debug_box(&mut lines, &terrain_translation);
    }
}

fn on_player_collision(
    collision_events: EventReader<CollisionEvent>,
    mut player: Query<&mut Transform, With<Player>>,
    mut scoreboard: ResMut<Scoreboard>,
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
) {
    let mut player_transform = player.single_mut();

    if !collision_events.is_empty() {
        scoreboard.score = 0;
        scoreboard.seconds_since_round_start = time.elapsed_seconds() as usize;

        player_transform.translation = DRAGON_STARTPOSITION;

        game_state.state = RunState::ResetTerrain;
    }
}

fn main() {
    //env::set_var("RUST_BACKTRACE","full");
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rusty Bird".into(),
                resolution: (800., 400.).into(),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(DebugLinesPlugin::default())
        .insert_resource(Scoreboard {
            score: 0,
            seconds_since_round_start: 0,
        })
        .insert_resource(GameState {
            state: RunState::Running,
        })
        .add_startup_system(setup)
        .add_event::<CollisionEvent>()
        .add_systems(
            (
                check_collisions,
                move_terrain_forward.before(check_collisions),
                player_movement_system
                    .before(check_collisions)
                    .after(move_terrain_forward),
                recycle_terrain.after(move_terrain_forward),
                simulated_gravity
                    .before(check_collisions)
                    .after(player_movement_system),
                on_player_collision.after(check_collisions),
                reset_terrain.after(on_player_collision),
            )
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .add_system(text_update_system)
        .add_system(update_scoreboard)
        .add_system(render_score)
        .add_system(draw_debug_lines)
        .run();
}
