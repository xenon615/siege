use std::time::Duration;

use bevy::prelude::*;
use avian3d::prelude::*;
use crate::shared::{Ball, GameLayer, Targetable, BallSpawn,Interval, BALL_DENSITY, BALL_RADIUS}; 

pub struct BallPlugin;
impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, startup)
        .add_systems(Update, despawn_on_time.run_if(any_with_component::<Interval>))
        .add_systems(Update, despawn_on_collision.run_if(on_event::<CollisionEnded>))
        .add_observer(spawn)
        ;

    }
}

// ---

#[derive(Resource)]
pub struct BallMM(Handle<Mesh>, Handle<StandardMaterial>);

// ---

fn startup (
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    cmd.insert_resource(BallMM(
        meshes.add(Sphere::new(BALL_RADIUS)),
        materials.add(Color::hsl(150., 1.0, 0.5))
    ));
}

// ---

fn spawn(
    trigger: Trigger<BallSpawn>,
    mut cmd: Commands,
    bmm: Res<BallMM> 
) {
    cmd.spawn((
        Mesh3d(bmm.0.clone()),
        MeshMaterial3d(bmm.1.clone()),
        Transform::from_translation(trigger.event().0),
        RigidBody::Dynamic,
        Restitution::new(0.).with_combine_rule(CoefficientCombine::Min),
        Collider::sphere(BALL_RADIUS),
        ColliderDensity(BALL_DENSITY),
        Ball,
        Name::new("Ball"),
        CollisionLayers::new(GameLayer::Targetable, [LayerMask::ALL])


    ));
}

// ---

fn despawn_on_time(
    mut t_q: Query<(Entity, &mut Interval), With<Ball>>,
    mut cmd: Commands,
    time: Res<Time>
) {
    for (e, mut interval) in & mut t_q {
        interval.0.tick(time.delta());
        if interval.0.finished() {
             cmd.entity(e).despawn();
        }
    }
}

// ---

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
