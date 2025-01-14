use std::time::Duration;

use bevy::prelude::*;
use avian3d::prelude::*;
use crate::shared::{Ball, GameLayer, Targetable, BallSpawn,Interval, BALL_DENSITY, BALL_RADIUS}; 

pub struct DBallPlugin;
impl Plugin for DBallPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, spawn)
        // .add_systems(Update, despawn_on_collision)
        ;

    }
}

// ---

fn spawn(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    let mesh = meshes.add(Sphere::new(BALL_RADIUS));
    let mat = materials.add(Color::hsl(100., 1.0, 0.5));
    for i in 0..2 {
        cmd.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(mat.clone()),
            Transform::from_xyz(-150. + i as f32 * 300., 50., -60.),
            RigidBody::Static,
            Collider::sphere(BALL_RADIUS),
            ColliderDensity(BALL_DENSITY),
            Ball,
            CollisionLayers::new(GameLayer::Targetable, [LayerMask::ALL]),
            Name::new("Ball")
        ));
    }

    cmd.spawn((
        Mesh3d(meshes.add(Cuboid::from_length(10.))),
        MeshMaterial3d(materials.add(Color::hsl(10., 1.0, 0.5))),
        Transform::from_xyz(0., 5., -0.),
        RigidBody::Static,
        Collider::cuboid(10., 10., 10.),
        Name::new("Cube"),
        // CollisionLayers::new(GameLayer::Targetable, [LayerMask::ALL])
    ));

}

// ---

#[allow(dead_code)]
fn despawn_on_collision(
    mut collision_events: EventReader<CollisionEnded>,
    mut t_q: Query<(Entity, &mut Interval), (With<Ball>, With<Targetable>)>,
) {
    for CollisionEnded(e1, e2) in collision_events.read() {
        for (e, mut i ) in &mut t_q {
            if e == *e1 || e == *e2 {
                i.0.set_duration(Duration::ZERO); 
            }
        }
    }
}
