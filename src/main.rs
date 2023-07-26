pub mod ground_detection;

use bevy::{
    app::AppExit,
    math::{vec2, vec3},
    prelude::*,
    render::camera::ScalingMode,
    sprite::MaterialMesh2dBundle,
};
use bevy_rapier2d::{na::ComplexField, prelude::*};
use bevy_rapier_collider_gen::*;
use std::f32::consts::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_state::<GameState>()
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        // .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .add_systems(Startup, setup)
        .add_systems(Update, (exit_on_esc, arrow_keys_apply_force))
        .add_systems(
            Update,
            generate_map_collider.run_if(in_state(GameState::Loading)),
        )
        .run();
}

#[derive(States, Clone, Eq, PartialEq, Debug, Default, Hash)]
enum GameState {
    #[default]
    Loading,
    Playing,
}

#[derive(Component)]
struct Player;

#[derive(Resource)]
struct MapImageHandle {
    collider_image: Handle<Image>,
    visual_image: Handle<Image>,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rapier_configuration: ResMut<RapierConfiguration>,
) {
    rapier_configuration.gravity = Vect::Y * (-9.81 * 10.0 * 10.0);
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 512.0,
                min_height: 512.0,
            },
            ..default()
        },
        ..default()
    });

    commands.insert_resource(MapImageHandle {
        collider_image: asset_server.load("col.png"),
        visual_image: asset_server.load("map.png"),
    });
}

#[derive(Component)]
pub struct TargetAngVel(pub Option<f32>);

fn arrow_keys_apply_force(
    keyboard_input: Res<Input<KeyCode>>,
    mut q_player: Query<(&mut Velocity, &mut TargetAngVel), With<Player>>,
    time: Res<Time>,
) {
    for (mut vel, mut target_angvel) in q_player.iter_mut() {
        let mut impulse = vec2(0.0, 0.0);
        let mut initial_target_vel = if let Some(target_angvel) = target_angvel.0 {
            target_angvel
        } else {
            vel.angvel
        };
        let mut newTargetAngVel = initial_target_vel;
        let mut explicit_angvel = false;
        if keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]) {
            newTargetAngVel += time.delta_seconds() * 8f32;
            explicit_angvel = true;
        }
        if keyboard_input.any_pressed([KeyCode::Right, KeyCode::D]) {
            newTargetAngVel -= time.delta_seconds() * 8f32;
            explicit_angvel = true;
        }
        if !explicit_angvel {
            target_angvel.0 = None;
        } else {
            if newTargetAngVel.abs() < initial_target_vel.abs() {
                newTargetAngVel *= 0.8f32;
            }
            newTargetAngVel = newTargetAngVel.clamp(-15f32, 15f32);
            target_angvel.0 = Some(newTargetAngVel);
            vel.angvel = dbg!(newTargetAngVel);
        }

        if keyboard_input.any_just_pressed([KeyCode::Space, KeyCode::W, KeyCode::Up]) {
            vel.linvel = Vect::new(vel.linvel.x, 360f32);
        }
    }
}

fn generate_map_collider(
    image_assets: ResMut<Assets<Image>>,
    map_image_handle: Option<Res<MapImageHandle>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    asset_server: Res<AssetServer>,
) {
    if let Some(map_image_handle) = map_image_handle {
        if let Some(collider_image) = image_assets.get(&map_image_handle.collider_image.clone()) {
            let colliders = multi_polyline_collider_translated(collider_image);

            // spawn map colliders
            for collider in colliders {
                commands.spawn((
                    collider,
                    RigidBody::Fixed,
                    SpriteBundle {
                        texture: map_image_handle.visual_image.clone(),
                        transform: Transform::from_xyz(0.0, 0.0, 0.0),
                        ..default()
                    },
                ));
            }

            // spawn player
            commands
                .spawn((Player, TargetAngVel(None)))
                .insert((
                    RigidBody::Dynamic,
                    Velocity::default(),
                    Ccd { enabled: true },
                ))
                .insert(Collider::ball(15.0))
                .insert(Restitution::coefficient(0.7))
                .insert(Friction::new(5.0))
                //.insert(AdditionalMassProperties::Mass(0.01f32))
                .insert(SpriteBundle {
                    texture: asset_server.load("player.png"),
                    transform: Transform::from_xyz(0.0, 20.0, 1.0),
                    sprite: Sprite {
                        custom_size: Some(vec2(30.0, 30.0)),
                        ..default()
                    },
                    ..default()
                });

            next_state.set(GameState::Playing);
        }
    }
}

fn exit_on_esc(keyboard_input: ResMut<Input<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}
