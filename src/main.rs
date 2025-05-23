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
            RigidBody::Dynamic,
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
) {
    let mut rng = rand::rng();
    let rand_loc_x = rng.random_range(-600.0..600.0);
    let rand_loc_y = rng.random_range(-600.0..600.0);

    if timer.0.tick(time.delta()).just_finished() {
        commands.spawn((
            Sprite::from(asset_server.load("drone.png")),
            Transform::from_xyz(rand_loc_x, rand_loc_y, 0.0).with_scale(Vec3::splat(1.)),
            RigidBody::Kinematic,
            Collider::circle(150.0),
            Sensor,
            CollidingEntities::default(),
            Object,
        ));
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
    mut q_colliding_entities: Query<(Entity, &CollidingEntities, &mut ExternalForce)>,
    q_player: Query<Entity, With<Player>>,
    q_object: Query<Entity, With<Object>>,
    q_transform: Query<&Transform>,
) {
    let players: Vec<Entity> = q_player.iter().collect();
    let objects: Vec<Entity> = q_object.iter().collect();
    for (entity, colliding_entities, mut force) in q_colliding_entities.iter_mut() {
        let coll_entis = colliding_entities.iter().collect::<Vec<_>>();
        for ent in coll_entis {
            if objects.contains(ent) && players.contains(&entity) {
                let player_transform = q_transform.get(entity).unwrap();
                let object_transform = q_transform.get(*ent).unwrap();
                let player_pos = player_transform.translation;
                let object_pos = object_transform.translation;
                let dir = (object_pos - player_pos).normalize();
                let x = dir.x * 25.;
                let y = dir.y * 25.;
                force.apply_force(Vec2 { x, y });
            }
        }
    }
}

fn consume_planets(
    q_objects: Query<(Entity, &Transform), With<Object>>,
    mut q_player: Query<&mut Transform, (With<Player>, Without<Object>)>,
    mut commands: Commands,
) {
    let mut player_transform = q_player.single_mut().unwrap();
    for (object, object_transform) in q_objects {
        if (player_transform.translation.x - object_transform.translation.x).abs() <= 25.0
            && (player_transform.translation.y - object_transform.translation.y).abs() <= 25.0
        {
            commands.entity(object.entity()).despawn();
            player_transform.translation.x = 0.;
            player_transform.translation.y = 0.;
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
        .add_systems(Update, detect_collisions)
        .add_systems(Update, quit_game)
        .add_systems(Update, consume_planets)
        .add_systems(Update, check_boundaries)
        .run();
}
