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
    cmd.spawn((
        DirectionalLight {
            illuminance: 500.,
            shadows_enabled: false,
            ..default()
        },
        Transform::IDENTITY.looking_at(Vec3::ZERO, Vec3::Y)
    ));
}

// ---

