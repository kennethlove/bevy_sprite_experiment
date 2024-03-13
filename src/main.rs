use bevy::prelude::*;
use bevy_editor_pls::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;
mod animation;
mod platforms;
mod player;

use animation::AnimationPlugin;
use platforms::PlatformsPlugin;
use player::PlayerPlugin;

const WINDOW_WIDTH: f32 = 1024.;
const WINDOW_HEIGHT: f32 = 720.;

pub const WINDOW_BOTTOM_Y: f32 = WINDOW_HEIGHT / -2.;
const WINDOW_LEFT_X: f32 = WINDOW_WIDTH / -2.;

const COLOR_BACKGROUND: Color = Color::DARK_GRAY;
const COLOR_FLOOR: Color = Color::GREEN;

const FLOOR_THICKNESS: f32 = 10.;

fn main() {
    App::new()
        .insert_resource(ClearColor(COLOR_BACKGROUND))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Sprite Experiment".to_string(),
                        resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        ) // prevents blurry sprites
        .add_plugins((
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(200.),
            RapierDebugRenderPlugin::default(),
        ))
        .add_plugins((WorldInspectorPlugin::new(), EditorPlugin::default()))
        .add_plugins((PlatformsPlugin, PlayerPlugin, AnimationPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: COLOR_FLOOR,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0., WINDOW_BOTTOM_Y - FLOOR_THICKNESS / 2., 0.),
                scale: Vec3::new(WINDOW_WIDTH, FLOOR_THICKNESS, 1.),
                ..default()
            },
            ..default()
        })
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(0.5, 0.5));
}
