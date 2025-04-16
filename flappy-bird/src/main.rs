use bevy::prelude::*;
use rand::Rng;

const WINDOW_RESOLUTION: Vec2 = Vec2::new(288., 512.);

const BIRD_WIDTH: f32 = 24.;
const BIRD_HEIGHT: f32 = 32.;
const GRAVITY: Vec2 = Vec2::new(0., -8.);
const JUMP_FORCE: Vec2 = Vec2::new(0., 5.);
const MIN_ROTATION: f32 = -std::f32::consts::FRAC_PI_3;
const MAX_ROTATION: f32 = std::f32::consts::FRAC_PI_3;

const PIPE_WIDTH: f32 = 52.;
const PIPE_HEIGHT: f32 = 320.;
const PIPE_SPAWN_INTERVAL: f32 = 2.0;
const GAP_HEIGHT: f32 = 100.;

struct Rect {
    min: Vec2,
    max: Vec2,
}

impl Rect {
    fn from_center_size(center: Vec2, size: Vec2) -> Self {
        Self {
            min: center - size / 2.,
            max: center + size / 2.,
        }
    }

    fn overlaps(&self, other: &Rect) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }
}

#[derive(Component)]
struct Bird {
    velocity: Vec2
}

#[derive(Component)]
struct Pipe {
    velocity: Vec2
}

#[derive(Resource)]
struct GameTextures {
    pipe: Handle<Image>,
    bird_down: Handle<Image>,
    bird_up: Handle<Image>
}

#[derive(Resource)]
struct PipeTimer(Timer);

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Flappy Bird".into(), 
                        resolution: WINDOW_RESOLUTION.into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
        )
        .add_systems(Startup, setup)
        .add_systems(Update, 
            (
                update_bird_system, 
                input_system, 
                spawn_pipes_system, 
                move_pipes_system,
                despawn_pipes_system,
                bird_collision_system
            )
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let background = asset_server.load("background.png");
    let pipe = asset_server.load("pipe.png");
    let bird_down = asset_server.load("bird-down.png");
    let bird_up = asset_server.load("bird-up.png");

    commands.insert_resource(GameTextures {
        pipe: pipe.clone(),
        bird_down: bird_down.clone(),
        bird_up: bird_up.clone()
    });
    
    commands.insert_resource(PipeTimer(Timer::from_seconds(PIPE_SPAWN_INTERVAL, TimerMode::Repeating)));
    
    commands.spawn(Camera2d);
    
    commands.spawn((
        Sprite::from_image(background),
        Transform::from_xyz(0., 0., 0.)
    ));

    commands.spawn((
        Sprite::from_image(bird_down),
        Transform::from_xyz(0., 0., 0.1),
        Bird { velocity: Vec2::new(0., 0.) }
    ));
}

fn input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut bird_query: Query<(&mut Bird, &mut Sprite)>,
    game_textures: Res<GameTextures>
) {
    let Ok((mut bird, mut bird_sprite)) = bird_query.get_single_mut() else { 
        return; 
    };

    if keys.just_pressed(KeyCode::Space) {
        bird.velocity.y = JUMP_FORCE.y;
        bird_sprite.image = game_textures.bird_up.clone();
    }
}

fn update_bird_system(
    time: Res<Time>,
    mut bird_query: Query<(&mut Bird, &mut Sprite, &mut Transform)>,
    game_textures: Res<GameTextures>
) {
    let dt = time.delta_secs();

    let Ok((mut bird, mut bird_sprite, mut bird_transform)) = bird_query.get_single_mut() else { 
        return; 
    };

    bird.velocity += GRAVITY * dt;
    bird_transform.translation += bird.velocity.extend(0.);
    bird_sprite.image = game_textures.bird_down.clone();
    
    let tilt_angle = bird.velocity.y * 0.05;
    let clamped_angle = tilt_angle.clamp(MIN_ROTATION, MAX_ROTATION);
    bird_transform.rotation = Quat::from_rotation_z(clamped_angle);
}

fn spawn_pipes_system(
    mut commands: Commands,
    time: Res<Time>,
    mut pipe_timer: ResMut<PipeTimer>,
    game_textures: Res<GameTextures>
) {
    if pipe_timer.0.tick(time.delta()).just_finished() {
        let mut rng = rand::rng();

        let gap_y = rng.random_range(-100.0 .. 100.0);
        
        let pipe_x = WINDOW_RESOLUTION.x / 2. + PIPE_WIDTH / 2. + 200.;
        let inf_pipe_y = gap_y - GAP_HEIGHT / 2. - PIPE_HEIGHT / 2.;
        let sup_pipe_y = gap_y + GAP_HEIGHT / 2. + PIPE_HEIGHT / 2.;

        commands.spawn((
            Sprite::from_image(game_textures.pipe.clone()),
            Transform::from_xyz(pipe_x, inf_pipe_y, 0.1),
            Pipe { velocity: Vec2::new(-3., 0.) }
        ));
        
        commands.spawn((
            Sprite::from_image(game_textures.pipe.clone()),
            Transform {
                translation: Vec3::new(pipe_x, sup_pipe_y, 0.1),
                rotation: Quat::from_rotation_z(std::f32::consts::PI),
                ..default()
            },
            Pipe { velocity: Vec2::new(-3., 0.) }
        ));
    }
}

fn move_pipes_system(mut pipe_query: Query<(&mut Transform, &Pipe)>) {
    for (mut pipe_transform, pipe) in pipe_query.iter_mut() {
        pipe_transform.translation += pipe.velocity.extend(0.);
    }
}

fn despawn_pipes_system(
    mut commands: Commands,
    pipe_query: Query<(Entity, &Transform), With<Pipe>>,
) {
    for (entity, transform) in pipe_query.iter() {
        if transform.translation.x < -WINDOW_RESOLUTION.x / 2. - PIPE_WIDTH / 2. {
            commands.entity(entity).despawn();
        }
    }
}

fn bird_collision_system(
    bird_query: Query<&Transform, With<Bird>>,
    pipe_query: Query<&Transform, With<Pipe>>,
    mut exit: EventWriter<AppExit>
) {
    let Ok(bird_transform) = bird_query.get_single() else {
        return;
    };
    
    let bird_size = Vec2::new(BIRD_WIDTH, BIRD_HEIGHT);
    let bird_pos = bird_transform.translation.truncate();
    let bird_rect = Rect::from_center_size(bird_pos, bird_size);

    for pipe_transform in pipe_query.iter() {
        let pipe_rect = {
            let pipe_size = Vec2::new(PIPE_WIDTH, PIPE_HEIGHT);
            let pipe_pos = pipe_transform.translation.truncate();
            Rect::from_center_size(pipe_pos, pipe_size)
        };

        if bird_rect.overlaps(&pipe_rect) ||
           bird_pos.y - bird_size.y / 2. <= -WINDOW_RESOLUTION.y / 2. || 
           bird_pos.y + bird_size.y / 2. >= WINDOW_RESOLUTION.y / 2.
        {
            exit.send(AppExit::Success);
        }
    }
}
