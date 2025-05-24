use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowMode};
use rand::Rng;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Turret;

#[derive(Component)]
struct Object;

#[derive(Component)]
struct Pickup;

#[derive(Component)]
struct Bullet;

#[derive(Resource)]
struct ObjectSpawnTimer(Timer);

#[derive(Resource)]
struct PickupSpawnTimer(Timer);

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let turret = commands.spawn((
            Sprite::from(asset_server.load("turret.png")),
            Transform::from_scale(Vec3::splat(1.)).with_translation(Vec3::new(0., 40., 100.)),
        )).id();
    let hing = commands.spawn((
            Transform::from_scale(Vec3::splat(1.)).with_translation(Vec3::splat(0.)),
            GlobalTransform::default(),
            Turret,
    )).id();
    let tank = commands
        .spawn((
            Sprite::from(asset_server.load("tank.png")),
            Transform::from_scale(Vec3::splat(0.75)),
            RigidBody::Kinematic,
            Collider::circle(50.0),
            CollidingEntities::default(),
            ExternalForce::new(Vec2::Y),
            Mass(5.0),
            Player,
        )).id();
    commands.entity(hing).add_children(&[turret]);
    commands.entity(tank).add_children(&[hing]);
}

fn move_player(
    mut q_player: Query<&mut LinearVelocity, With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let mut velocity = q_player.single_mut().unwrap();
    let mut x = 0.0;
    let mut y = 0.0;
    if keys.pressed(KeyCode::KeyA) {
        x -= 100.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        x += 100.0;
    }
    if keys.pressed(KeyCode::KeyW) {
        y += 100.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        y -= 100.0;
    }
    velocity.x = x;
    velocity.y = y;
}

fn rotate_player(mut q_player: Query<(&mut LinearVelocity, &mut Transform), With<Player>>) {
    if let Ok((velocity, mut transform)) = q_player.single_mut() {
        let v = Vec2{x: velocity.x, y: velocity.y};
        if v.length_squared() > 0.01 {
            let angle = v.y.atan2(v.x);
            transform.rotation = Quat::from_rotation_z(angle - std::f32::consts::FRAC_PI_2);
        }
    }
}

pub fn rotate_turret(
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut q_turret: Query<&mut Transform, (With<Turret>, Without<Player>)>,
    q_player: Query<&GlobalTransform, (With<Player>, Without<Turret>)>,
) {
    let window = match q_window.single() {
        Ok(win) => win,
        Err(_) => return,
    };
    let (camera, camera_transform) = match q_camera.single() {
        Ok(val) => val,
        Err(_) => return,
    };
    let mut turret = match q_turret.single_mut() {
        Ok(val) => val,
        Err(_) => return,
    };
    let player_transform = match q_player.single() {
        Ok(val) => val,
        Err(_) => return,
    };
    let cursor_screen_pos = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_screen_pos) else {
        return;
    };

    let player_pos = player_transform.translation().truncate();
    let dir = (world_pos - player_pos).normalize();
    let angle = dir.y.atan2(dir.x) - std::f32::consts::FRAC_PI_2;
    let parent_angle = player_transform.to_scale_rotation_translation().1.to_euler(EulerRot::ZYX).0;
    turret.rotation = Quat::from_rotation_z(angle - parent_angle);
}


fn spawn_objects(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut timer: ResMut<ObjectSpawnTimer>,
    q_window: Query<&Window, With<PrimaryWindow>>,
) {
    let mut rng = rand::rng();
    let rand_loc_x = rng.random_range(10.0..250.0);
    let rand_loc_y = rng.random_range(10.0..250.0);
    let rand_select_x = rng.random_range(0..=1) as usize;
    let rand_select_y = rng.random_range(0..=1) as usize;
    let win = q_window.single().unwrap();
    let y_positive = win.height() + rand_loc_y;
    let x_positive = win.width()+ rand_loc_x;
    let y_negative = (win.height() * -1.) - rand_loc_y;
    let x_negative = (win.width() * -1.) - rand_loc_x;
    let xs = [x_negative, x_positive];
    let ys = [y_negative, y_positive];

    if timer.0.tick(time.delta()).just_finished() {
        commands.spawn((
            Sprite::from(asset_server.load("drone.png")),
            Transform::from_xyz(xs[rand_select_x], ys[rand_select_y], 0.0).with_scale(Vec3::splat(1.)),
            RigidBody::Kinematic,
            Collider::circle(10.0),
            Sensor,
            CollidingEntities::default(),
            Object,
        ));
    }
}

fn move_objects (
    q_object: Query<&mut Transform, (With<Object>, Without<Player>)>,
    q_player: Query<&Transform, With<Player>>,
) {
    let transform_player = match q_player.single() {
        Ok(k) => k,
        Err(_e) => return,
    };
    for mut transform_object in q_object {
        let direction = (transform_object.translation - transform_player.translation).normalize() * -1.;
        transform_object.translation.x += direction.x;
        transform_object.translation.y += direction.y;
    }
}

#[allow(clippy::too_many_arguments)]
    pub fn fire_bullet(
        mut commands: Commands,
        keyboard_input: Res<ButtonInput<KeyCode>>,
        mouse_input: Res<ButtonInput<MouseButton>>,
        asset_server: Res<AssetServer>,
        mut q_player: Query<&mut Transform, With<Player>>,
        q_windows: Query<&Window, With<PrimaryWindow>>,
        q_camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    ) {
        let (camera, camera_transform) = match q_camera.single() {
            Ok(k) => k,
            Err(_e) => return,
        };
        if keyboard_input.just_pressed(KeyCode::Space)
            || mouse_input.just_pressed(MouseButton::Left)
        {
            let win = q_windows.single().unwrap();
            let position = win.cursor_position().unwrap();
            let Ok(world_pos_cursor) = camera.viewport_to_world_2d(camera_transform, position) else {return;};
            for transform in q_player.iter_mut() {
                let pos = Vec2::from((
                    world_pos_cursor.x,
                    world_pos_cursor.y,
                ));
                let dir = (pos - transform.translation.truncate()).normalize();
                let angle = dir.y.atan2(dir.x);
                commands
                    .spawn((
                        Sprite::from_image(asset_server.load("bullet.png")),
                        Transform::from_translation(transform.translation)
                            .with_scale(Vec3::splat(1.))
                            .with_rotation(Quat::from_rotation_z(angle - std::f32::consts::FRAC_PI_2)),
                        Bullet,
                        RigidBody::Kinematic,
                        Collider::circle(10.),
                        CollidingEntities::default(),
                        //Sensor,
                    ));
                    
                // let bullet_fire_entity = commands
                //     .spawn((
                //         AudioPlayer::new(asset_server.load("fire.ogg")),
                //         PlaybackSettings::ONCE,
                //     ))
                //     .id();
            }
        }
    }
    
fn move_bullet(
        mut query: Query<&mut Transform, With<Bullet>>,
        time: Res<Time>,
    ) {
        let time_step = time.delta_secs();
        for mut transform in query.iter_mut() {
            let up_direction = transform.up();
            transform.translation += 1000. * time_step * up_direction;
        }
    }

#[allow(dead_code)]
fn spawn_pickups(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut timer: ResMut<PickupSpawnTimer>,
) {
    let mut rng = rand::rng();
    let rand_loc_x = rng.random_range(-500.0..500.0);
    let rand_loc_y = rng.random_range(-500.0..500.0);

    if timer.0.tick(time.delta()).just_finished() {
        commands.spawn((
            Sprite::from(asset_server.load("pickup.png")),
            Transform::from_xyz(rand_loc_x, rand_loc_y, 0.0).with_scale(Vec3::splat(1.)),
            RigidBody::Kinematic,
            Collider::circle(10.0),
            Sensor,
            CollidingEntities::default(),
            Pickup,
        ));
    }
}

fn detect_collisions(
    mut q_colliding_entities: Query<(Entity, &CollidingEntities)>,
    q_player: Query<Entity, With<Player>>,
    q_object: Query<Entity, With<Object>>,
    q_bullet: Query<Entity, With<Bullet>>,
    mut commands: Commands,
) {
    let players: Vec<Entity> = q_player.iter().collect();
    let objects: Vec<Entity> = q_object.iter().collect();
    let bullets: Vec<Entity> = q_bullet.iter().collect();
    for (entity, colliding_entities) in q_colliding_entities.iter_mut() {
        let coll_entis = colliding_entities.iter().collect::<Vec<_>>();
        for ent in coll_entis {
            if objects.contains(ent) && players.contains(&entity) {
                println!("Game Over");
            }
            if bullets.contains(ent) && objects.contains(&entity) || bullets.contains(&entity) && objects.contains(ent){
                commands.entity(*ent).despawn();
                commands.entity(entity).despawn();
            }
        }
    }
}

#[allow(dead_code)]
fn hide_and_lock_cursor(mut q_window: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = q_window.single_mut().unwrap();
    window.cursor_options.visible = false;
}

fn quit_game(keys: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

#[allow(dead_code)]
fn spawn_bg(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Sprite::from_image(asset_server.load("bg.png")),
        Transform::from_scale(Vec3::splat(0.75)).with_translation(Vec3 {
            x: 0.,
            y: 0.,
            z: -10.,
        }),
    ));
}

#[allow(dead_code)]
fn start_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioPlayer::new(asset_server.load("music.ogg")),
        PlaybackSettings::LOOP,
    ));
}

fn check_boundaries(
    mut q_player: Query<&mut Transform, With<Player>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
) {
    let win = q_window.single().unwrap();
    let mut transform = q_player.single_mut().unwrap();
    let y = win.height();
    let x = win.width();
    if transform.translation.x > x / 2. {
        transform.translation.x = -1. * x / 2. + 1.0;
    }
    if transform.translation.x < -1. * x / 2. {
        transform.translation.x = x / 2. - 1.0;
    }
    if transform.translation.y > y / 2. {
        transform.translation.y = -1. * y / 2. + 1.0;
    }
    if transform.translation.y < -1. * y / 2. {
        transform.translation.y = y / 2. - 1.0;
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(PhysicsPlugins::default())
        //.add_plugins(PhysicsDebugPlugin::default())
        .insert_resource(Gravity(Vec2::splat(0.)))
        .insert_resource(ObjectSpawnTimer(Timer::from_seconds(
            5.0,
            TimerMode::Repeating,
        )))
        .insert_resource(PickupSpawnTimer(Timer::from_seconds(
            8.0,
            TimerMode::Repeating,
        )))
        //.add_systems(Startup, spawn_bg)
        //.add_systems(Startup, start_music)
        //.add_systems(Startup, hide_and_lock_cursor)
        .add_systems(Startup, spawn_camera)
        .add_systems(Startup, spawn_player)
        .add_systems(Update, move_player)
        .add_systems(Update, rotate_player)
        .add_systems(Update, rotate_turret)
        .add_systems(Update, spawn_objects)
        .add_systems(Update, move_objects)
        .add_systems(Update, detect_collisions)
        .add_systems(Update, fire_bullet)
        .add_systems(Update, move_bullet)
        .add_systems(Update, quit_game)
        .add_systems(Update, check_boundaries)
        .run();
}
