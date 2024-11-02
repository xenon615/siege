use bevy::prelude::*;
use avian3d::prelude::{Collider, RigidBody};
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut al: ResMut<AmbientLight>
) {
    al.brightness = 100.;
    // cmd.spawn((
    //     PbrBundle {
    //         material: materials.add(Color::BLACK),
    //         mesh: meshes.add(Plane3d::new(Vec3::Y, Vec2::new(100., 100.))),
    //         ..default()
    //     },
    //     RigidBody::Static,
    //     Collider::cuboid(200., 0.1, 200.)
    // ));

    cmd.spawn(DirectionalLightBundle{

        ..default()
    });
}

// ---

