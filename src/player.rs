use std::time::Duration;

use crate::animation::{Animation, AnimationIndices, AnimationTimer};
use crate::WINDOW_BOTTOM_Y;
use crate::WINDOW_LEFT_X;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

const SPRITESHEET_COLS: usize = 8;
const SPRITESHEET_ROWS: usize = 9;
const SPRITE_TILE_WIDTH: f32 = 32.;
const SPRITE_TILE_HEIGHT: f32 = 32.;
const SPRITE_RENDER_WIDTH: f32 = 128.;
const SPRITE_RENDER_HEIGHT: f32 = 128.;
const SPRITE_IDX_STAND: usize = 0;
const SPRITE_IDX_IDLE: &[usize] = &[8, 9];
const SPRITE_IDX_WALK: &[usize] = &[16, 17, 18, 19];
const SPRITE_IDX_JUMP: &[usize] = &[40, 41, 42];
const SPRITE_IDX_FALL: &[usize] = &[43, 44, 45, 46, 47];
const IDLE_CYCLE_DELAY: Duration = Duration::from_millis(250);
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

#[derive(Clone, Component, Debug, Default, Eq, Hash, PartialEq, States)]
enum ActionState {
    #[default]
    Idle,
    Walk,
    Jump,
    Fall,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<ActionState>()
            .add_systems(Startup, setup)
            .add_systems(FixedUpdate, (
                idle,
                movement,
                // jump,
                // rise,
                fall,
                update_direction
            ))
            .add_systems(
                Update,
                (
                    log_transitions,
                    apply_idle_animation,
                    apply_movement_animation,
                    // apply_rise_sprite,
                    // apply_fall_sprite,
                    // update_direction,
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
    .insert(KinematicCharacterController {
        snap_to_ground: Some(CharacterLength::Relative(0.1)),
        ..default()
    })
    .insert(Direction::Right)
    .insert(Animation::new(
        SPRITE_IDX_IDLE,
        IDLE_CYCLE_DELAY,
    ));
    // .insert(AnimationIndices {
    //     first: SPRITE_IDX_IDLE[0],
    //     last: SPRITE_IDX_IDLE[1],
    // })
    // .insert(AnimationTimer(Timer::new(IDLE_CYCLE_DELAY, TimerMode::Repeating)));
}

fn log_transitions(mut transitions: EventReader<StateTransitionEvent<ActionState>>) {
    for transition in transitions.read() {
        info!(
            "transition: {:?} => {:?}",
            transition.before, transition.after
        );
    }
}

fn idle(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut KinematicCharacterController)>,
    mut state: ResMut<NextState<ActionState>>,
) {
    let (entity, mut player) = query.single_mut();
    state.set(ActionState::Idle);

    // let mut movement = 0.0;

    // if movement != 0.0 {
    //     movement = 0.0;
    // }

    // match player.translation {
    //     Some(vec) => {
    //         player.translation = Some(Vec2::new(movement, vec.y))
    //     },
    //     None => {
    //         player.translation = Some(Vec2::new(movement, 0.))
    //     },
    // }
}

fn movement(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(Entity, &mut KinematicCharacterController)>,
    mut state: ResMut<NextState<ActionState>>,
) {
    let (entity, mut player) = query.single_mut();

    let mut movement = 0.0;

    state.set(ActionState::Idle);
    if input.pressed(KeyCode::ArrowRight) {
        state.set(ActionState::Walk);
        movement += time.delta_seconds() * PLAYER_VELOCITY_X;
    }
    if input.pressed(KeyCode::ArrowLeft) {
        state.set(ActionState::Walk);
        movement -= time.delta_seconds() * PLAYER_VELOCITY_X;
    }

    match player.translation {
        Some(vec) => {
            player.translation = Some(Vec2::new(movement, vec.y))
        },
        None => {
            player.translation = Some(Vec2::new(movement, 0.))
        },
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

const WALK_CYCLE_INDICES: AnimationIndices = AnimationIndices {
    first: 16, last: 19,
};
fn apply_movement_animation(
    mut commands: Commands,
    state: Res<State<ActionState>>,
    mut query: Query<(Entity, &KinematicCharacterControllerOutput, &mut TextureAtlas)>,
) {
    if query.is_empty() {
        return;
    }
    if state.get() != &ActionState::Walk {
        return;
    }

    let (player, output, mut sprite) = query.single_mut();

    if output.desired_translation.x != 0.0 && output.grounded {
    // if output.grounded {
        info!("applying walk animation");
        commands
            .entity(player)
            .insert(Animation::new(
                SPRITE_IDX_WALK,
                WALK_CYCLE_DELAY
            ));
            sprite.index = *SPRITE_IDX_WALK.first().unwrap();

        // if sprite.index < *SPRITE_IDX_WALK.last().unwrap() {
        //     sprite.index += 1;
        // } else {
        //     sprite.index = *SPRITE_IDX_WALK.first().unwrap();
        // }
    }
}

fn apply_idle_animation(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &KinematicCharacterControllerOutput,
        &mut TextureAtlas,
        &Animation,
    )>,
    state: Res<State<ActionState>>,
    // ), Without<Animation>>,
) {
    if query.is_empty() {
        return;
    }
    if state.get() != &ActionState::Idle {
        return;
    }

    let (player, output, mut sprite, animation) = query.single_mut();

    if output.desired_translation.x == 0.0 && output.grounded {
    // if output.grounded {
        info!("applying idle animation");
        commands
            .entity(player)
                .insert(Animation::new(
                    SPRITE_IDX_IDLE,
                    IDLE_CYCLE_DELAY,
                ));
        // sprite.index = SPRITE_IDX_IDLE[0];
                // sprite.index += 1;
                // .insert(AnimationIndices {
                //     first: 0,
                //     last: 1,
                // })
                // .insert(AnimationTimer(Timer::new(IDLE_CYCLE_DELAY, TimerMode::Repeating)));
        // sprite.index = 0;
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
