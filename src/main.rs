use std::f32::consts::PI;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle, render::{settings::{WgpuSettings, Backends}, RenderPlugin}};

const BALL_RADIUS: f32 = 10.0;
const BALL_SPEED: f32 = 7.0;

const PADDLE_WIDTH: f32 = 10.0;
const PADDLE_HEIGHT: f32 = 80.0;
const PADDLE_SHAPE: Vec2 = Vec2 { x: PADDLE_WIDTH, y: PADDLE_HEIGHT};
const PADDLE_SPEED: f32 = 4.0;

const BOARD_WIDTH: f32 = 600.0;
const BOARD_HEIGHT: f32 = 400.0;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct LeftScoreDisplay;

#[derive(Component)]
struct RightScoreDisplay;

#[derive(Component)]
struct AI;

#[derive(Component)]
struct PlayerControls {
    up: KeyCode,
    down: KeyCode,
}

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Resource)]
struct Score {
    left: i32,
    right: i32,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: bevy::render::settings::RenderCreation::Automatic(WgpuSettings {
                backends: Some(Backends::VULKAN),
                ..default()}),
            }))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            move_paddle,
            start_game,
            collision,
            move_ball,
            score,
        ));
    
    if std::env::args().nth(1) == Some("-2p".to_string()) {
        app.add_systems(Startup, setup_2_player);
    } else {
        app.add_systems(Startup, setup_ai);
        app.add_systems(Update, play_ai);
    }
    
    app.run();
}

fn setup_2_player(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Quad::new(PADDLE_SHAPE).into()).into(),
            material: materials.add(ColorMaterial::from(Color::WHITE)),
            transform: Transform::from_translation(Vec3 { x: (-BOARD_WIDTH + PADDLE_WIDTH)/2.0, y: 0.0, z: 0.0 }),
            ..default()
        },
        Paddle,
        PlayerControls { up: KeyCode::W, down: KeyCode::S },
    ));
}

fn setup_ai(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Quad::new(PADDLE_SHAPE).into()).into(),
            material: materials.add(ColorMaterial::from(Color::WHITE)),
            transform: Transform::from_translation(Vec3 { x: (-BOARD_WIDTH + PADDLE_WIDTH)/2.0, y: 0.0, z: 0.0 }),
            ..default()
        },
        Paddle,
        AI,
    ));
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(BALL_RADIUS).into()).into(),
            material: materials.add(ColorMaterial::from(Color::WHITE)),
            ..default()
        },
        Ball,
        Velocity(Vec2::ZERO),
    ));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Quad::new(PADDLE_SHAPE).into()).into(),
            material: materials.add(ColorMaterial::from(Color::WHITE)),
            transform: Transform::from_translation(Vec3 { x: (BOARD_WIDTH - PADDLE_WIDTH)/2.0, y: 0.0, z: 0.0 }),
            ..default()
        },
        Paddle,
        PlayerControls { up: KeyCode::Up, down: KeyCode::Down },
    ));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Quad::new(Vec2 { x: BOARD_WIDTH, y: BOARD_HEIGHT}).into()).into(),
            material: materials.add(ColorMaterial::from(Color::DARK_GRAY)),
            transform: Transform::from_translation(Vec3 {x: 0.0, y: 0.0, z: -2.0}),
            ..default()
        },
    ));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Quad::new(Vec2 { x: BOARD_WIDTH + 20.0, y: BOARD_HEIGHT + 20.0}).into()).into(),
            material: materials.add(ColorMaterial::from(Color::GRAY)),
            transform: Transform::from_translation(Vec3 {x: 0.0, y: 0.0, z: -3.0}),
            ..default()
        },
    ));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Quad::new(Vec2 { x: 10.0, y: BOARD_HEIGHT}).into()).into(),
            material: materials.add(ColorMaterial::from(Color::GRAY)),
            transform: Transform::from_translation(Vec3 {x: 0.0, y: 0.0, z: -1.0}),
            ..default()
        },
    ));


    commands.insert_resource(Score { left: 0, right: 0 });

    commands.spawn((
        TextBundle::from_section(
                "0",
                TextStyle {
                    font_size: 60.0,
                    color: Color::GRAY,
                    ..default()
                },
            ).with_style(Style {
            margin: UiRect {
                left: Val::Vw(25.),
                right: Val::Percent(0.),
                top: Val::Percent(5.),
                bottom: Val::Percent(0.)
            },
            ..Default::default()
        }),
        LeftScoreDisplay
    ));

    commands.spawn((
        TextBundle::from_section(
            "0",
            TextStyle {
                font_size: 60.0,
                color: Color::GRAY,
                ..default()
            },
        ).with_style(Style {
            margin: UiRect {
                left: Val::Vw(75.),
                right: Val::Percent(0.),
                top: Val::Percent(5.),
                bottom: Val::Percent(0.)
            },
            ..Default::default()
        }),
        RightScoreDisplay
    ));
}

fn move_paddle(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &PlayerControls), (With<Paddle>, Without<AI>)>,
) {
    for (mut transform, controls) in query.iter_mut() {
        let mut direction = 0.0;
        
        if keyboard_input.pressed(controls.up) {
            direction += 1.0;
        } 
        if keyboard_input.pressed(controls.down) {
            direction -= 1.0;
        }
        
        let new_transform = (transform.translation.y + direction * PADDLE_SPEED).clamp(BOARD_HEIGHT / -2.0 + PADDLE_HEIGHT / 2.0, BOARD_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0);
        transform.translation.y = new_transform;
    }

}

fn move_ball(mut query: Query<(&mut Transform, &Velocity), With<Ball>>) {
    let (mut transform, velocity) = query.single_mut();
    transform.translation += velocity.0.extend(0.0);
}

fn collision(
    mut ball_query: Query<(&Transform, &mut Velocity), With<Ball>>,
    paddle_query: Query<&Transform, With<Paddle>>,
) {
    let (ball_transform, mut ball_velocity) = ball_query.single_mut();
    let ball_pos = ball_transform.translation;
    for paddle_transform in paddle_query.iter() {
        let paddle_pos = paddle_transform.translation;
        let reflection_angle = (ball_pos.y - paddle_pos.y) * PI / (PADDLE_HEIGHT * 2.0);
        let distance = paddle_pos.x - ball_pos.x;
        
        if (ball_pos.y - paddle_pos.y).abs() <= PADDLE_HEIGHT / 2.0 + BALL_RADIUS {
            if distance <= PADDLE_WIDTH/2.0 + BALL_RADIUS && distance > 0.0 {
                ball_velocity.0 = Vec2::new(-BALL_SPEED * reflection_angle.cos(), BALL_SPEED * reflection_angle.sin());
            } else if distance >= -(PADDLE_WIDTH/2.0 + BALL_RADIUS) && distance < 0.0 {
                ball_velocity.0 = Vec2::new(BALL_SPEED * reflection_angle.cos(), BALL_SPEED * reflection_angle.sin());
            }
        }
    }

    if ball_pos.y >= BOARD_HEIGHT/2.0 - BALL_RADIUS || ball_pos.y <= -BOARD_HEIGHT/2.0 + BALL_RADIUS {
        ball_velocity.0.y *= -1.0;
    }
}

fn start_game(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Transform), With<Ball>>,
) {
    if keyboard_input.just_pressed(KeyCode::Return) {
        let (mut velocity, mut transform) = query.single_mut();
        velocity.0 = Vec2::new(BALL_SPEED, 0.0);
        transform.translation = Vec3::ZERO;
    }
}

fn score(
    mut query: Query<(&mut Velocity, &mut Transform), With<Ball>>,
    mut left_query: Query<&mut Text, (With<LeftScoreDisplay>, Without<RightScoreDisplay>)>,
    mut right_query: Query<&mut Text, (Without<LeftScoreDisplay>, With<RightScoreDisplay>)>,
    mut score: ResMut<Score>,
) {
    let (mut velocity, mut transform) = query.single_mut();
    if transform.translation.x >= BOARD_WIDTH/2.0 - BALL_RADIUS {
        score.as_mut().left += 1;
        velocity.0 = Vec2::ZERO;
        transform.translation = Vec3::ZERO;
        left_query.single_mut().sections[0].value = score.left.to_string();
    } else if transform.translation.x <= -BOARD_WIDTH/2.0 + BALL_RADIUS {
        score.as_mut().right += 1;
        velocity.0 = Vec2::ZERO;
        transform.translation = Vec3::ZERO;
        right_query.single_mut().sections[0].value = score.right.to_string();
    }
}

fn play_ai(
    ball_query: Query<&Transform, (With<Ball>, Without<AI>)>,
    mut ai_query:  Query<&mut Transform, With<AI>>,
) {
    let ball_transform = ball_query.single();
    let mut ai_transform = ai_query.single_mut();

    let direction = (ball_transform.translation.y - ai_transform.translation.y).clamp(-PADDLE_SPEED, PADDLE_SPEED);

    let new_translation = (ai_transform.translation.y + direction).clamp(BOARD_HEIGHT / -2.0 + PADDLE_HEIGHT / 2.0, BOARD_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0);
    ai_transform.translation.y = new_translation;
}