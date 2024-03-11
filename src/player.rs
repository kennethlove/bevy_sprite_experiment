use std::time::Duration;

use crate::animation::Animation;
use crate::WINDOW_BOTTOM_Y;
use crate::WINDOW_LEFT_X;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

const SPRITESHEET_COLS: usize = 8;
const SPRITESHEET_ROWS: usize = 9;
const SPRITE_TILE_WIDTH: f32 = 32.;
const SPRITE_TILE_HEIGHT: f32 = 32.;
const SPRITE_RENDER_WIDTH: f32 = 64.;
const SPRITE_RENDER_HEIGHT: f32 = 64.;
const SPRITE_IDX_STAND: usize = 0;
const SPRITE_IDX_IDLE: &[usize] = &[0, 1];
const SPRITE_IDX_WALK: &[usize] = &[16, 17, 18, 19];
const SPRITE_IDX_JUMP: &[usize] = &[40, 41, 42];
const SPRITE_IDX_FALL: &[usize] = &[43, 44, 45, 46, 47];
const IDLE_CYCLE_DELAY: Duration = Duration::from_millis(2000);
const WALK_CYCLE_DELAY: Duration = Duration::from_millis(500);
const RISE_CYCLE_DELAY: Duration = Duration::from_millis(700);
const FALL_CYCLE_DELAY: Duration = Duration::from_millis(700);
const PLAYER_VELOCITY_X: f32 = 400.;
const PLAYER_VELOCITY_Y: f32 = 450.;

const MAX_JUMP_HEIGHT: f32 = 230.;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
struct Jump(f32);

#[derive(Component)]
enum Direction {
    Left,
    Right,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (
                movement,
                jump,
                rise,
                fall,
                apply_idle_animation,
                // apply_movement_animation,
                // apply_rise_sprite,
                // apply_fall_sprite,
                update_direction,
                update_sprite_direction,
            ),
        );
    }
}

fn setup(
    mut commands: Commands,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
) {
    let texture: Handle<Image> = asset_server.load("AnimationSheet_Character.png");
    let texture_atlas = TextureAtlasLayout::from_grid(
        Vec2::new(SPRITE_TILE_WIDTH, SPRITE_TILE_HEIGHT),
        SPRITESHEET_COLS,
        SPRITESHEET_ROWS,
        None,
        None,
    );
    let atlas_handle = atlases.add(texture_atlas);

    commands.spawn(SpriteSheetBundle {
        sprite: Sprite::default(),
        atlas: TextureAtlas {
            layout: atlas_handle,
            index: SPRITE_IDX_IDLE[0],
        },
        texture,
        transform: Transform {
            translation: Vec3::new(WINDOW_LEFT_X + 100., WINDOW_BOTTOM_Y + 300., 0.),
            scale: Vec3::new(
                SPRITE_RENDER_WIDTH / SPRITE_TILE_WIDTH,
                SPRITE_RENDER_HEIGHT / SPRITE_TILE_HEIGHT,
                1.,
            ),
            ..default()
        },
        ..default()
    })
    .insert(RigidBody::KinematicPositionBased)
    .insert(Collider::cuboid(
        SPRITE_TILE_WIDTH / 2.,
        SPRITE_TILE_HEIGHT / 2.,
    ))
    .insert(KinematicCharacterController::default())
    .insert(Direction::Right);
}

fn movement(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut KinematicCharacterController>,
) {
    let mut player = query.single_mut();

    let mut movement = 0.0;

    if input.pressed(KeyCode::ArrowRight) {
        movement += time.delta_seconds() * PLAYER_VELOCITY_X;
    }
    if input.pressed(KeyCode::ArrowLeft) {
        movement -= time.delta_seconds() * PLAYER_VELOCITY_X;
    }

    match player.translation {
        Some(vec) => player.translation = Some(Vec2::new(movement, vec.y)),
        None => player.translation = Some(Vec2::new(movement, 0.)),
    }
}

fn jump(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    query: Query<
        (Entity, &KinematicCharacterControllerOutput),
        (With<KinematicCharacterController>, Without<Jump>),
    >,
) {
    if query.is_empty() {
        return;
    }

    let (player, output) = query.single();

    if input.pressed(KeyCode::ArrowUp) && output.grounded {
        commands.entity(player).insert(Jump(0.));
    }
}

fn rise(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut KinematicCharacterController, &mut Jump)>,
) {
    if query.is_empty() {
        return;
    }

    let (entity, mut player, mut jump) = query.single_mut();

    let mut movement = time.delta().as_secs_f32() * PLAYER_VELOCITY_Y;

    if movement + jump.0 >= MAX_JUMP_HEIGHT {
        movement = MAX_JUMP_HEIGHT - jump.0;
        commands.entity(entity).remove::<Jump>();
    }

    jump.0 += movement;

    match player.translation {
        Some(vec) => player.translation = Some(Vec2::new(vec.x, movement)),
        None => player.translation = Some(Vec2::new(0.0, movement)),
    }
}

fn fall(time: Res<Time>, mut query: Query<&mut KinematicCharacterController, Without<Jump>>) {
    if query.is_empty() {
        return;
    }

    let mut player = query.single_mut();

    let movement = time.delta().as_secs_f32() * (PLAYER_VELOCITY_Y / 2.5) * -1.;

    match player.translation {
        Some(vec) => player.translation = Some(Vec2::new(vec.x, movement)),
        None => player.translation = Some(Vec2::new(0.0, movement)),
    }
}

fn apply_movement_animation(
    mut commands: Commands,
    query: Query<(Entity, &KinematicCharacterControllerOutput), Without<Animation>>,
) {
    if query.is_empty() {
        return;
    }

    let (player, output) = query.single();
    if output.desired_translation.x != 0.0 && output.grounded {
        info!("applying walk animation");
        commands
            .entity(player)
            .insert(Animation::new(SPRITE_IDX_WALK, WALK_CYCLE_DELAY));
    }
}

fn apply_idle_animation(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &KinematicCharacterControllerOutput,
        // &mut TextureAtlas,
    ), With<TextureAtlas>>,
    // ), Without<Animation>>,
) {
    if query.is_empty() {
        return;
    }

    let (player, output) = query.single_mut();
    if output.desired_translation.x == 0.0 && output.grounded {
        info!("applying idle animation");
        commands
            .entity(player)
            .insert(Animation::new(SPRITE_IDX_IDLE, IDLE_CYCLE_DELAY));
        // sprite.index = SPRITE_IDX_IDLE[0];
    }
}

fn apply_rise_sprite(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &KinematicCharacterControllerOutput,
            &mut TextureAtlas,
        ),
        With<Jump>,
    >,
) {
    if query.is_empty() {
        return;
    }

    let (player, output, mut sprite) = query.single_mut();
    if !output.grounded && output.desired_translation.y > 0. {
        info!("applying rise sprite");
        commands
            .entity(player)
            .insert(Animation::new(SPRITE_IDX_JUMP, RISE_CYCLE_DELAY));
        // sprite.index = SPRITE_IDX_JUMP[2];
    }
}

fn apply_fall_sprite(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &KinematicCharacterControllerOutput,
            &mut TextureAtlas,
        ),
        With<Jump>,
    >,
) {
    if query.is_empty() {
        return;
    }

    let (player, output, mut sprite) = query.single_mut();
    if !output.grounded && output.desired_translation.y < 0. {
        info!("applying fall sprite");
        info!("{}", output.grounded);
        commands
            .entity(player)
            .insert(Animation::new(SPRITE_IDX_FALL, FALL_CYCLE_DELAY));
        // sprite.index = SPRITE_IDX_FALL[0];
    }
}
fn update_direction(
    mut commands: Commands,
    query: Query<(Entity, &KinematicCharacterControllerOutput)>,
) {
    if query.is_empty() {
        return;
    }

    let (player, output) = query.single();

    if output.desired_translation.x > 0. {
        commands.entity(player).insert(Direction::Right);
    } else if output.desired_translation.x < 0. {
        commands.entity(player).insert(Direction::Left);
    }
}

fn update_sprite_direction(mut query: Query<(&mut Sprite, &Direction)>) {
    if query.is_empty() {
        return;
    }

    let (mut sprite, direction) = query.single_mut();
    match direction {
        Direction::Right => {
            sprite.flip_x = false;
        }
        Direction::Left => {
            sprite.flip_x = true;
        }
    }
}
