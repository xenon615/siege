use bevy::prelude::*;
use bevy::core_pipeline::Skybox;
use bevy::render::camera::{Exposure, PhysicalCameraParameters};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

// ---

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, spawn) 
        .add_plugins(PanOrbitCameraPlugin)
        ;
    }
} 

// ---

#[derive(Component)]
pub struct Cam;

// ---

fn spawn (
    mut commands : Commands,
    assets: ResMut<AssetServer>
) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-20., 5., 0.).looking_at(Vec3::ZERO, Vec3::Y),
            exposure: Exposure::from_physical_camera(PhysicalCameraParameters {
                sensitivity_iso: 80.,
                ..default()
            }),
            camera: Camera {
                hdr: true,
                ..default()
            },
            ..default()
        },
        Skybox {
            image: assets.load("skyboxes/space_green.ktx2"),
            brightness: 500.,
        },
        PanOrbitCamera {
            enabled: true,
            focus: Vec3::new(10., 5., 0.),
            ..default()
        },
        Cam,
    ));
}

// ---
