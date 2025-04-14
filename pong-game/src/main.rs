use bevy::prelude::*;

const WINDOW_WIDTH: f32 = 800.;
const WINDOW_HEIGHT: f32 = 600.;

const PADDLE_1_COLOR: Color = Color::srgb(0.3, 0.7, 0.3);
const PADDLE_2_COLOR: Color = Color::srgb(0.3, 0.3, 0.7);

const PADDLE_SIZE: Vec2 = Vec2::new(100., 10.);
const PADDLE_OFFSET: f32 = 20.;
const PADDLE_VELOCITY: Vec3 = Vec3::new(400., 0., 0.);

const BALL_COLOR: Color = Color::srgb(0.7, 0.3, 0.3);

const BALL_SIZE: Vec2 = Vec2::new(10., 10.);
const BALL_VELOCITY: Vec3 = Vec3::new(300., 300., 0.);

#[derive(Component)]
struct Paddle {
    player: u8
}

#[derive(Component)]
struct Ball;

#[derive(Component, Clone)]
struct Velocity(Vec3);

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Pong Game".into(),
                        resolution: Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
        )
        .insert_resource(ClearColor(Color::srgb(0.9, 0.9, 0.9)))
        .add_systems(Startup, setup)
        .add_systems(Update, (input_system, ball_movement_system, wall_collision_system, paddle_collision_system))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite {
            color: PADDLE_1_COLOR,
            custom_size: Some(PADDLE_SIZE),
            ..default()
        },
        Transform::from_xyz(0., WINDOW_HEIGHT/2. - PADDLE_OFFSET, 0.),
        Paddle { player: 1 }
    ));

    commands.spawn((
        Sprite {
            color: PADDLE_2_COLOR,
            custom_size: Some(PADDLE_SIZE),
            ..default()
        },
        Transform::from_xyz(0., -WINDOW_HEIGHT/2. + PADDLE_OFFSET, 0.),
        Paddle { player: 2 }
    ));
    
    commands.spawn((
        Sprite {
            color: BALL_COLOR,
            custom_size: Some(BALL_SIZE),
            ..default()
        },
        Transform::from_xyz(0., 0., 0.),
        Ball,
        Velocity(BALL_VELOCITY),
    ));
}

fn input_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>, 
    mut query: Query<(&mut Transform, &Paddle)>
) {
    let dt = time.delta_secs();

    for (mut transform, paddle) in query.iter_mut() {
        match paddle.player {
            1 => {
                if keys.pressed(KeyCode::KeyA) && transform.translation.x > -WINDOW_WIDTH / 2. + PADDLE_SIZE.x / 2. {
                    transform.translation -= PADDLE_VELOCITY * dt;
                } else if keys.pressed(KeyCode::KeyD) && transform.translation.x < WINDOW_WIDTH / 2. - PADDLE_SIZE.x / 2. {
                    transform.translation += PADDLE_VELOCITY * dt;
                }
            },
            2 => {
                if keys.pressed(KeyCode::ArrowLeft) && transform.translation.x > -WINDOW_WIDTH / 2. + PADDLE_SIZE.x / 2. {
                    transform.translation -= PADDLE_VELOCITY * dt;
                } else if keys.pressed(KeyCode::ArrowRight) && transform.translation.x < WINDOW_WIDTH / 2. - PADDLE_SIZE.x / 2. {
                    transform.translation += PADDLE_VELOCITY * dt;
                }
            },
            _ => {}
        }
    }
}

fn ball_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity), With<Ball>>,
) {
    let dt = time.delta_secs();

    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0 * dt;
    }
}

fn wall_collision_system(
    mut ball_query: Query<(&mut Transform, &mut Velocity), With<Ball>>,
) {
    let (ball_transform, mut velocity) = ball_query.single_mut();

    if ball_transform.translation.x < -WINDOW_WIDTH / 2. + BALL_SIZE.x / 2.
        || ball_transform.translation.x > WINDOW_WIDTH / 2. - BALL_SIZE.x / 2.
    {
        velocity.0.x *= -1.;
    }

    if ball_transform.translation.y < -WINDOW_HEIGHT / 2. + BALL_SIZE.y / 2.
        || ball_transform.translation.y > WINDOW_HEIGHT / 2. - BALL_SIZE.y / 2.
    {
        velocity.0.y *= -1.;
    }
}

fn paddle_collision_system(
    mut ball_query: Query<(&Transform, &mut Velocity), With<Ball>>,
    paddle_query: Query<&Transform, With<Paddle>>,
) {
    let (ball_transform, mut velocity) = ball_query.single_mut();

    for paddle_transform in paddle_query.iter() {
        if aabb_collision(ball_transform.translation, BALL_SIZE, paddle_transform.translation, PADDLE_SIZE) {
            velocity.0.y *= -1.;
            return;
        }
    }
}

fn aabb_collision(a_pos: Vec3, a_size: Vec2, b_pos: Vec3, b_size: Vec2) -> bool {
    let collision_x = (a_pos.x - b_pos.x).abs() < (a_size.x + b_size.x) / 2.;
    let collision_y = (a_pos.y - b_pos.y).abs() < (a_size.y + b_size.y) / 2.;
    collision_x && collision_y
}

