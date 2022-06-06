A basic fps-style flycamera for bevy

# Controls
* WASD to move
* LCTRL to descend
* Space to ascend
* Escape to unlock cursor

The controls are customizable

# Usage
1. Add to Cargo.toml, matching major/minor with bevy
```toml
[dependencies]
bevy = "X.Y"
bevy-fpscam = "X.Y"
```

2. Use the plugin
```rust
use bevy_fpscam::FpsCamPlugin;
```
This will spawn the camera for you. If you want to create
the camera yourself, use `NoSpawnFpsCamPlugin` instead, and
add a `FpsCam` component to your camera.

3. Add the plugin
```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(FpsCamPlugin)
        .run();     
}
```

 # Customization
 You can modify mouse sensitivity, movement speed and keybindings
 by modifying the resource of type `bevy_fpscam::Config`
 ```rust
 fn main() {
     App::new()
         .add_plugins(DefaultPlugins)
         .add_plugin(FpsCamPlugin)
         .insert_resource(bevy_fpscam::Config{
             movespeed: 2.0,
             sensitivity: 0.01,
             key_bindings: KeyBindings {
                 unlock: Some(KeyCode::Enter),
                 ..Default::default()
         }}).run();
 }
 ```