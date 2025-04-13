use bevy::prelude::*;
use bevy::ecs::system::ParamSet;
use rand::Rng;

const WINDOW_WIDTH: f32 = 800.;
const WINDOW_HEIGHT: f32 = 600.;

const FOOD_SIZE: Vec2 = Vec2::new(10., 10.);
const FOOD_START_POSITION: Vec2 = Vec2::new(50., 50.);
const FOOD_COLOR: Color = Color::srgb(0.7, 0.3, 0.3);

const SNAKE_SIZE: Vec2 = Vec2::new(10., 10.);
const SNAKE_COLOR: Color = Color::srgb(0.3, 0.3, 0.7);
const SNAKE_SPEED: f32 = 200.;

#[derive(Component)]
struct Food;

#[derive(Component)]
struct SnakeSegment;

#[derive(Resource)]
struct Direction(Vec2);

#[derive(Resource)]
struct Snake(Vec<Entity>);

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Snake Game".into(), 
                        resolution: Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
        )
        .insert_resource(ClearColor(Color::srgb(0.9, 0.9, 0.9)))
        .insert_resource(Direction(Vec2::X))
        .add_systems(Startup, setup)
        .add_systems(Update, (snake_input_system, snake_movement_system, food_collision_system, self_collision_system))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    
    let center = Vec2::ZERO;

    commands.spawn((
        Sprite {
            color: FOOD_COLOR,
            custom_size: Some(FOOD_SIZE),
            ..default()
        },
        Transform::from_translation(FOOD_START_POSITION.extend(0.)),
        Food,
    ));

    let mut snake = Vec::new();
    for i in 0..3 {
        let pos = center + Vec2::new(-i as f32 * SNAKE_SIZE.x, 0.);
        let entity = commands
            .spawn((
                Sprite {
                    color: SNAKE_COLOR,
                    custom_size: Some(SNAKE_SIZE),
                    ..default()
                },
                Transform::from_translation(pos.extend(0.)),
                SnakeSegment,
            ))
            .id();
        snake.push(entity);
    }

    commands.insert_resource(Snake(snake));
}

fn snake_input_system(keys: Res<ButtonInput<KeyCode>>, mut dir: ResMut<Direction>) {
    if keys.pressed(KeyCode::ArrowUp) && dir.0 != -Vec2::Y {
        dir.0 = Vec2::Y;
    } else if keys.pressed(KeyCode::ArrowDown) && dir.0 != Vec2::Y {
        dir.0 = -Vec2::Y;
    } else if keys.pressed(KeyCode::ArrowLeft) && dir.0 != Vec2::X {
        dir.0 = -Vec2::X;
    } else if keys.pressed(KeyCode::ArrowRight) && dir.0 != -Vec2::X {
        dir.0 = Vec2::X;
    }
}

fn snake_movement_system(
    time: Res<Time>,
    dir: Res<Direction>,
    snake: Res<Snake>,
    mut query: Query<&mut Transform, With<SnakeSegment>>,
) {
    let dt = time.delta_secs();

    let mut previous_positions: Vec<Vec3> = Vec::new();

    for &entity in snake.0.iter() {
        if let Ok(transform) = query.get(entity) {
            previous_positions.push(transform.translation);
        }
    }

    if let Ok(mut head_transform) = query.get_mut(snake.0[0]) {
        head_transform.translation += (dir.0 * SNAKE_SPEED * dt).extend(0.0);
    }

    for (i, &entity) in snake.0.iter().enumerate().skip(1) {
        if let Ok(mut transform) = query.get_mut(entity) {
            transform.translation = previous_positions[i - 1];
        }
    }
}

fn food_collision_system(
    mut commands: Commands,
    mut snake: ResMut<Snake>,
    mut param_set: ParamSet<(
        Query<&mut Transform, With<SnakeSegment>>,
        Query<(Entity, &Transform), With<Food>>,
    )>,
) {
    let head_position = {
        let mut segment_query = param_set.p0();
        let Ok(head_transform) = segment_query.get_mut(snake.0[0]) else {
            return;
        };
        head_transform.translation.truncate()
    };

    let food_data: Vec<(Entity, Vec2)> = param_set
        .p1()
        .iter()
        .map(|(e, transform)| (e, transform.translation.truncate()))
        .collect();

    for (food_entity, food_pos) in food_data {
        let distance = head_position.distance(food_pos);

        if distance < (SNAKE_SIZE.x / 2.0 + FOOD_SIZE.x / 2.0) {
            commands.entity(food_entity).despawn();

            if let Some(&last_segment) = snake.0.last() {
                let segment_query = param_set.p0();
                if let Ok(last_transform) = segment_query.get(last_segment) {
                    let new_segment = commands
                        .spawn((
                            Sprite {
                                color: SNAKE_COLOR,
                                custom_size: Some(SNAKE_SIZE),
                                ..default()
                            },
                            Transform::from_translation(last_transform.translation),
                            SnakeSegment,
                        ))
                        .id();
                    snake.0.push(new_segment);
                }
            }
            
            spawn_food(&mut commands);
        }
    }
}

fn spawn_food(commands: &mut Commands) {
    let x_range = -WINDOW_WIDTH / 2. .. WINDOW_WIDTH / 2.;
    let y_range = -WINDOW_HEIGHT / 2. .. WINDOW_HEIGHT / 2.;

    let mut rng = rand::rng();
    let random_x = rng.random_range(x_range.clone());
    let random_y = rng.random_range(y_range.clone());

    let random_pos = Vec3::new(random_x, random_y, 0.0);

    commands.spawn((
        Sprite {
            color: FOOD_COLOR,
            custom_size: Some(FOOD_SIZE),
            ..default()
        },
        Transform::from_translation(random_pos),
        Food,
    ));
}

fn self_collision_system(
    snake: Res<Snake>,
    query: Query<&Transform, With<SnakeSegment>>,
    mut exit: EventWriter<AppExit>,
) {
    if snake.0.len() < 4 {
        return;
    }

    let head_pos = {
        let Ok(head_transform) = query.get(snake.0[0]) else { 
            return; 
        };
        head_transform.translation.truncate()
    };

    for &segment in &snake.0[1..] {
        if let Ok(segment_transform) = query.get(segment) {
            let segment_pos = segment_transform.translation.truncate();
            let distance = head_pos.distance(segment_pos);

            if distance < SNAKE_SIZE.x / 2.0 {
                exit.send(AppExit::Success);
            }
        }
    }
}
