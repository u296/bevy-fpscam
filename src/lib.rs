//! A basic fps-style flycamera for bevy
//!
//! # Controls
//! * WASD to move
//! * LCTRL to descend
//! * Space to ascend
//! * Escape to unlock cursor
//!
//! The controls are customizable
//!
//! # Usage
//! 1. Add to Cargo.toml, matching major/minor with bevy
//! ```
//! [dependencies]
//! bevy = "X.Y"
//! bevy-fpscam = "X.Y"
//! ```
//!
//! 2. Use the plugin
//! ```
//! use bevy_fpscam::FpsCamPlugin;
//! ```
//! This will spawn the camera for you. If you want to create
//! the camera yourself, use `NoSpawnFpsCamPlugin` instead, and
//! add a `FpsCam` component to your camera.
//!
//! 3. Add the plugin
//! ```
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(FpsCamPlugin)
//!         .run();     
//! }
//! ```
//!
//! # Customization
//! You can modify mouse sensitivity, movement speed and keybindings
//! by modifying the resource of type `bevy_fpscam::Config`
//! ```
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(FpsCamPlugin)
//!         .insert_resource(bevy_fpscam::Config{
//!             movespeed: 2.0,
//!             sensitivity: 0.01,
//!             key_bindings: KeyBindings {
//!                 unlock: Some(KeyCode::Enter),
//!                 ..Default::default()
//!         }}).run();
//! }
//! ```

use bevy::{
    input::{
        keyboard::KeyboardInput,
        mouse::{MouseButtonInput, MouseMotion},
        ElementState,
    },
    prelude::*,
    window::WindowFocused,
};

/// Keybindings for controlling the camera. Default is WASD for movement, space
/// for up, LCTRL for down and ESC for unlocking the cursor. All keybinds are
/// optional.
#[derive(Clone, Copy, Debug)]
pub struct KeyBindings {
    pub forward: Option<KeyCode>,
    pub back: Option<KeyCode>,
    pub left: Option<KeyCode>,
    pub right: Option<KeyCode>,
    pub up: Option<KeyCode>,
    pub down: Option<KeyCode>,
    pub unlock: Option<KeyCode>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            forward: Some(KeyCode::W),
            back: Some(KeyCode::S),
            left: Some(KeyCode::A),
            right: Some(KeyCode::D),
            up: Some(KeyCode::Space),
            down: Some(KeyCode::LControl),
            unlock: Some(KeyCode::Escape),
        }
    }
}

/// Global configuration for the camera. modify the resource of this
/// type to change from the default configuration
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub movespeed: f32,
    pub sensitivity: f32,
    pub key_bindings: KeyBindings,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            movespeed: 1.0,
            sensitivity: 0.001,
            key_bindings: Default::default(),
        }
    }
}

/// Represents the player controlled camera. Attaching this to an entity which
/// has a transform will make it controllable by the player. Note that if you
/// put this component on multiple entities they will all be controlled
/// simultaneously by the player.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct FpsCam {
    pub yaw: f32,
    pub pitch: f32,
}

/// Handles camera movement
fn camera_move(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    config: Res<Config>,
    windows: Res<Windows>,
    mut q: Query<&mut Transform, With<FpsCam>>,
) {
    let window = windows.get_primary().unwrap();
    for mut transform in q.iter_mut() {
        let mut v = Vec3::ZERO;

        let forward = transform.forward();
        let right = transform.right();

        if window.cursor_locked() {
            for key in keys.get_pressed() {
                match Some(*key) {
                    x if x == config.key_bindings.forward => v += forward,
                    x if x == config.key_bindings.back => v -= forward,
                    x if x == config.key_bindings.left => v -= right,
                    x if x == config.key_bindings.right => v += right,
                    x if x == config.key_bindings.up => v += Vec3::Y,
                    x if x == config.key_bindings.down => v -= Vec3::Y,

                    _ => (),
                }
            }
        }

        v = v.normalize_or_zero();

        transform.translation += v * time.delta_seconds() * config.movespeed;
    }
}

/// Handles camera looking, only when the cursor is locked
fn camera_look(
    config: Res<Config>,
    windows: Res<Windows>,
    mut motion: EventReader<MouseMotion>,
    mut q: Query<(&mut Transform, &mut FpsCam)>,
) {
    let window = windows.get_primary().unwrap();
    for (mut transform, mut fpscam) in q.iter_mut() {
        for event in motion.iter() {
            if window.cursor_locked() {
                fpscam.yaw -= config.sensitivity * event.delta.x;
                fpscam.pitch -= config.sensitivity * event.delta.y;

                fpscam.pitch = fpscam
                    .pitch
                    .clamp(-std::f32::consts::PI / 2.0, std::f32::consts::PI / 2.0);

                transform.rotation = Quat::from_axis_angle(Vec3::Y, fpscam.yaw)
                    * Quat::from_axis_angle(Vec3::X, fpscam.pitch);
            }
        }
    }
}

/// Handles matching the cursor lock state when the window gains or loses focus
fn lock_on_focus(mut windows: ResMut<Windows>, mut focus_events: EventReader<WindowFocused>) {
    let window = windows.get_primary_mut().unwrap();
    for ev in focus_events.iter() {
        if ev.id == window.id() {
            set_cursor_lock(window, ev.focused);
        }
    }
}

/// Handles unlocking the cursor when the key is pressed
fn unlock_cursor(
    config: Res<Config>,
    mut windows: ResMut<Windows>,
    mut key_events: EventReader<KeyboardInput>,
) {
    let window = windows.get_primary_mut().unwrap();
    for kev in key_events.iter() {
        if let Some(code) = kev.key_code {
            if Some(code) == config.key_bindings.unlock {
                set_cursor_lock(window, false);
            }
        }
    }
}

/// Handles locking the cursor when the client area is clicked
fn lock_cursor(mut windows: ResMut<Windows>, mut mouse_events: EventReader<MouseButtonInput>) {
    let window = windows.get_primary_mut().unwrap();
    for ev in mouse_events.iter() {
        if ev.state == ElementState::Pressed {
            set_cursor_lock(window, true);
        }
    }
}

/// Spawns the camera
fn spawn_camera(mut cmd: Commands) {
    cmd.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(1.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    })
    .insert(FpsCam::default());
}

fn set_cursor_lock(window: &mut Window, state: bool) {
    window.set_cursor_lock_mode(state);
    window.set_cursor_visibility(!state);
}

/// Spawns a camera and sets up the controls.
pub struct FpsCamPlugin;
impl Plugin for FpsCamPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Config>()
            .add_startup_system(spawn_camera)
            .add_system(camera_move)
            .add_system(camera_look)
            .add_system(lock_on_focus)
            .add_system(lock_cursor)
            .add_system(unlock_cursor);
    }
}

/// Sets up the controls, but does not actually spawn a camera.
pub struct NoSpawnFpsCamPlugin;
impl Plugin for NoSpawnFpsCamPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Config>()
            .add_system(camera_move)
            .add_system(camera_look)
            .add_system(lock_on_focus)
            .add_system(lock_cursor)
            .add_system(unlock_cursor);
    }
}
