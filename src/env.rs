use bevy::prelude::*;

// ---

pub struct EnvPlugin;
impl Plugin for EnvPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, startup)
        ;
    }
}

// ---

fn startup(
    mut cmd: Commands,
    // mut al: ResMut<AmbientLight>
) {
    // al.brightness = 250.;
    cmd.spawn(DirectionalLightBundle{
        directional_light: DirectionalLight {
            illuminance: 500.,
            ..default()
        },
        ..default()
    });
}

// ---

