use std::time::Duration;

use crate::animation::{AnimationIndices, AnimationTimer};
use crate::WINDOW_BOTTOM_Y;
use crate::WINDOW_LEFT_X;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

const SPRITE_SHEET_COLS: usize = 8;
const SPRITE_SHEET_ROWS: usize = 9;
const SPRITE_TILE_WIDTH: f32 = 32.;
const SPRITE_TILE_HEIGHT: f32 = 32.;
const SPRITE_RENDER_WIDTH: f32 = 128.;
const SPRITE_RENDER_HEIGHT: f32 = 128.;
const SPRITE_INDICES_IDLE: AnimationIndices = AnimationIndices { first: 0, last: 1 };
const SPRITE_INDICES_BLINK: AnimationIndices = AnimationIndices { first: 8, last: 9 };
const SPRITE_INDICES_WALK: AnimationIndices = AnimationIndices {
    first: 16,
    last: 19,
};
const SPRITE_INDICES_RUN: AnimationIndices = AnimationIndices { first: 24, last: 31 };
const SPRITE_INDICES_RISE: AnimationIndices = AnimationIndices {
    first: 41,
    last: 42,
};
const SPRITE_INDICES_FALL: AnimationIndices = AnimationIndices {
    first: 43,
    last: 47,
};
const IDLE_CYCLE_DELAY: Duration = Duration::from_millis(350);
const WALK_CYCLE_DELAY: Duration = Duration::from_millis(50);
const RUN_CYCLE_DELAY: Duration = Duration::from_millis(500);
const RISE_CYCLE_DELAY: Duration = Duration::from_millis(250);
const FALL_CYCLE_DELAY: Duration = Duration::from_millis(700);
const PLAYER_WALK_VELOCITY_X: f32 = 100.;
const PLAYER_RUN_VELOCITY_X: f32 = 350.;
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
    Setup,
    Idle,
    Walk,
    Run,
    Jump,
    Fall,
}

#[derive(Resource)]
struct PlayerSpriteSheet(Handle<TextureAtlasLayout>);

impl FromWorld for PlayerSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let texture_atlas = TextureAtlasLayout::from_grid(
            Vec2::new(SPRITE_TILE_WIDTH, SPRITE_TILE_HEIGHT),
            SPRITE_SHEET_COLS,
            SPRITE_SHEET_ROWS,
            None,
            None,
        );

        let mut texture_atlases = world
            .get_resource_mut::<Assets<TextureAtlasLayout>>()
            .unwrap();
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        Self(texture_atlas_handle)
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ActionState>()
            .init_resource::<PlayerSpriteSheet>()
            .add_systems(Startup, setup)
            .add_systems(
                FixedUpdate,
                (
                    idle,
                    movement.after(idle),
                    jump.after(idle),
                    rise.after(jump),
                    fall.after(rise),
                    update_direction,
                ),
            )
            .add_systems(
                Update,
                (
                    log_transitions,
                    apply_idle_animation.run_if(in_state(ActionState::Idle)),
                    apply_walk_animation.run_if(in_state(ActionState::Walk)),
                    apply_run_animation.run_if(in_state(ActionState::Run)),
                    apply_rise_sprite.run_if(in_state(ActionState::Jump)),
                    apply_fall_sprite.run_if(in_state(ActionState::Fall)),
                    update_sprite_direction,
                ),
            );
    }
}

fn setup(
    mut commands: Commands,
    sprite_atlas: Res<PlayerSpriteSheet>,
    asset_server: Res<AssetServer>,
    mut state: ResMut<NextState<ActionState>>,
) {
    let sprite: Handle<Image> = asset_server.load("AnimationSheet_Character.png");

    commands
        .spawn(SpriteSheetBundle {
            sprite: Sprite::default(),
            atlas: TextureAtlas {
                layout: sprite_atlas.0.clone(),
                index: SPRITE_INDICES_FALL.first,
            },
            texture: sprite,
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
        .insert(Direction::Right)
        .insert(SPRITE_INDICES_FALL)
        .insert(AnimationTimer(Timer::new(
            IDLE_CYCLE_DELAY,
            TimerMode::Repeating,
        )));
    state.set(ActionState::Fall)
}

fn log_transitions(mut transitions: EventReader<StateTransitionEvent<ActionState>>) {
    for transition in transitions.read() {
        info!(
            "transition: {:?} => {:?}",
            transition.before, transition.after
        );
    }
}

fn idle(mut state: ResMut<NextState<ActionState>>) {
    state.set(ActionState::Idle);
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

    if input.pressed(KeyCode::ArrowRight) {
        if input.pressed(KeyCode::ShiftLeft) {
            movement += time.delta_seconds() * PLAYER_RUN_VELOCITY_X;
            state.set(ActionState::Run);
        } else {
            movement += time.delta_seconds() * PLAYER_WALK_VELOCITY_X;
            state.set(ActionState::Walk);
        }
    }
    if input.pressed(KeyCode::ArrowLeft) {
        if input.pressed(KeyCode::ShiftLeft) {
            movement -= time.delta_seconds() * PLAYER_RUN_VELOCITY_X;
            state.set(ActionState::Run);
        } else {
            movement -= time.delta_seconds() * PLAYER_WALK_VELOCITY_X;
            state.set(ActionState::Walk);
        }
    }

    match player.translation {
        Some(vec) => player.translation = Some(Vec2::new(movement, vec.y)),
        None => player.translation = Some(Vec2::new(movement, 0.)),
    }
}

fn jump(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<NextState<ActionState>>,
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
        state.set(ActionState::Jump);
        commands.entity(player).insert(Jump(0.));
    }
}

fn rise(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<NextState<ActionState>>,
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
        state.set(ActionState::Fall);
    }

    jump.0 += movement;

    match player.translation {
        Some(vec) => player.translation = Some(Vec2::new(vec.x, movement)),
        None => player.translation = Some(Vec2::new(0.0, movement)),
    }
}

fn fall(
    time: Res<Time>,
    mut state: ResMut<NextState<ActionState>>,
    mut query: Query<&mut KinematicCharacterController, Without<Jump>>,
) {
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

fn apply_walk_animation(
    mut commands: Commands,
    mut query: Query<(Entity, &KinematicCharacterControllerOutput)>,
) {
    if query.is_empty() {
        return;
    }

    let (player, output) = query.single_mut();

    if output.desired_translation.x != 0.0 && output.grounded {
        info!("applying walk animation");
        commands.entity(player).insert(SPRITE_INDICES_WALK);
    }
}

fn apply_run_animation(
    mut commands: Commands,
    mut query: Query<(Entity, &KinematicCharacterControllerOutput)>,
) {
    if query.is_empty() {
        return;
    }

    let (player, output) = query.single_mut();

    if output.desired_translation.x != 0.0 && output.grounded {
        info!("applying run animation");
        commands.entity(player).insert(SPRITE_INDICES_RUN);
    }
}

fn apply_idle_animation(
    mut commands: Commands,
    mut query: Query<(Entity, &KinematicCharacterControllerOutput)>,
) {
    if query.is_empty() {
        return;
    }

    let (player, output) = query.single_mut();

    if output.desired_translation.x == 0.0 && output.grounded {
        info!("applying idle animation");

        commands.entity(player).insert(SPRITE_INDICES_IDLE);
    }
}

fn apply_rise_sprite(
    mut commands: Commands,
    mut query: Query<(Entity, &KinematicCharacterControllerOutput), With<Jump>>,
) {
    if query.is_empty() {
        return;
    }

    let (player, output) = query.single_mut();
    if !output.grounded && output.desired_translation.y > 0. {
        info!("applying rise sprite");
        commands
            .entity(player)
            .insert(SPRITE_INDICES_RISE)
            .insert(AnimationTimer(Timer::new(
                RISE_CYCLE_DELAY,
                TimerMode::Repeating,
            )));
    }
}

fn apply_fall_sprite(
    mut commands: Commands,
    mut query: Query<(Entity, &KinematicCharacterControllerOutput), Without<Jump>>,
    mut state: ResMut<NextState<ActionState>>,
) {
    if query.is_empty() {
        return;
    }

    let (player, output) = query.single_mut();
    if !output.grounded && output.desired_translation.y < 0. {
        info!("applying fall sprite");
        state.set(ActionState::Fall);
        commands.entity(player).insert(SPRITE_INDICES_FALL);
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
