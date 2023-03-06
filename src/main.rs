use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use copy_to_output::copy_to_output;
use leafwing_input_manager::prelude::*;

#[derive(Component)]
struct Player;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Move,
}

const MOVE_FORCE: f32 = 1500.0;
fn movement(
    mut query: Query<(&ActionState<Action>, &mut ExternalForce), With<Player>>,
    time: Res<Time>,
) {
    for (action_state, mut external_force) in &mut query {
        let axis_vector = action_state.axis_pair(Action::Move).unwrap().xy();
        external_force.force = axis_vector * MOVE_FORCE * time.delta_seconds();
    }
}

fn collision_sounds(
    rapier_context: Res<RapierContext>,
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
) {
    let mut is_collided = false;

    for pair in rapier_context.contact_pairs() {
        if pair.has_any_active_contacts() {
            is_collided = true
        }
    }

    if is_collided {
        let sound = asset_server.load("impact.ogg");
        audio.play(sound);
    }
}

fn spawn_player(
    id: usize,
    location: Vec2,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    // Add player
    commands
        .spawn(SpriteBundle {
            texture: asset_server.load(if id == 0 {
                "red_ball.png"
            } else {
                "blue_ball.png"
            }),
            transform: Transform::from_translation(location.extend(1.0)),
            ..default()
        })
        .insert(InputManagerBundle::<Action> {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(DualAxis::left_stick(), Action::Move)
                .insert(
                    if id == 0 {
                        VirtualDPad::wasd()
                    } else {
                        VirtualDPad::arrow_keys()
                    },
                    Action::Move,
                )
                .set_gamepad(Gamepad { id })
                .build(),
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(15.0))
        .insert(ExternalForce {
            force: Vec2::ZERO,
            torque: 0.0,
        })
        .insert(Damping {
            linear_damping: 0.6,
            angular_damping: 5.0,
        })
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Restitution::coefficient(1.0))
        .insert(Player);
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Add 2D camera
    commands.spawn(Camera2dBundle::default());

    // Add player 1
    spawn_player(0, Vec2::new(-150.0, 0.0), &mut commands, &asset_server);
    // Add player 2
    spawn_player(1, Vec2::new(150.0, 0.0), &mut commands, &asset_server);

    // Add triangle
    commands
        .spawn(SpriteBundle {
            transform: Transform::from_xyz(150.0, 200.0, 0.0),
            texture: asset_server.load("block.png"),
            ..default()
        })
        .insert(Collider::cuboid(15.0, 15.0))
        .insert(RigidBody::Fixed)
        .insert(Restitution::coefficient(1.0));
}

fn main() {
    copy_to_output("assets", "release").expect("error ocurred when move assets folder to target");
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                height: 800.0,
                title: "Rolling".to_string(),
                width: 600.0,
                ..default()
            },
            ..default()
        }))
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(200.0))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            ..default()
        })
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup)
        .add_system(movement)
        .add_system(collision_sounds)
        .run();
}
