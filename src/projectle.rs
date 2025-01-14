use std::{collections::HashMap, time::Duration};

use bevy::{
    prelude::*,
    pbr::{NotShadowCaster, NotShadowReceiver}
};

use avian3d::prelude::*;
use crate::shared::GameLayer; 

pub struct ProjectlePlugin;
impl Plugin for ProjectlePlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, startup)
        .add_systems(Update, despawn_on_time.run_if(any_with_component::<LifeTime>))
        .add_systems(Update, despawn_on_collision.run_if(on_event::<CollisionEnded>))
        .add_observer(spawn)
        ;
    }
}

// ---
#[derive(Component)]
pub struct  Ball;

#[derive(Component)]
pub struct  Bullet;

#[derive(Component)]
pub struct  Projectle;

#[derive(Component)]
pub struct  Released;

#[derive(Component)]
pub struct LifeTime(pub Timer);

pub struct ProjectleMM(Handle<Mesh>, Handle<StandardMaterial>);

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum ProjectleKey {
    Ball,
    Bullet
}
#[derive(Resource)]
pub struct Projectles(HashMap<ProjectleKey, ProjectleMM>);

#[derive(Event)]
pub struct ProjectleSpawn {
    pub key: ProjectleKey,
    pub pos: Vec3,
    pub dir: Option<Dir3>,
    pub impulse: Option<Vec3>,
    pub lifetime: Option<u64>
}

// ---

pub const BALL_DENSITY: f32 = 14.5;
pub const BALL_RADIUS: f32 = 0.55;

pub const BULLET_DENSITY: f32 = 1.;
pub const BULLET_RADIUS: f32 = 0.5;

// ---


fn startup (
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

) {
    cmd.insert_resource(Projectles(
        HashMap::from([
            (
                ProjectleKey::Ball,
                ProjectleMM(
                    meshes.add(Sphere::new(BALL_RADIUS)),
                    materials.add(Color::hsl(150., 1.0, 0.5))
                )
            ),
            (
                ProjectleKey::Bullet,
                ProjectleMM(
                    meshes.add(Sphere::new(BULLET_RADIUS)),
                    // materials.add(Color::hsl(0., 1.0, 0.5))
                    materials.add(StandardMaterial {
                        emissive: LinearRgba::from(Color::hsl(47., 1.0, 0.5)),
                        ..default()
                    })

                )
            )

        ])
    ));
}

// ---

fn spawn(
    trigger: Trigger<ProjectleSpawn>,
    mut cmd: Commands,
    projectles: Res<Projectles> 
) {
    let event = trigger.event();
    let conf = projectles.0.get(&event.key).unwrap();
    let id = cmd.spawn((
        Mesh3d(conf.0.clone()),
        MeshMaterial3d(conf.1.clone()),
        RigidBody::Dynamic,
        Restitution::new(0.).with_combine_rule(CoefficientCombine::Min),
        NotShadowCaster,
        NotShadowReceiver,
        Projectle,
    
    ))
    .id()
    ;
    
    if event.key == ProjectleKey::Ball {
        cmd.entity(id).insert((
            Collider::sphere(BALL_RADIUS), ColliderDensity(BALL_DENSITY), Ball, Name::new("Ball"), CollisionLayers::new(GameLayer::Attacker, [LayerMask::ALL]),
        ));
    } else {
        cmd.entity(id).insert((
            Collider::sphere(BULLET_RADIUS), ColliderDensity(BULLET_DENSITY), Bullet, Name::new("Bullet"), CollisionLayers::new(GameLayer::Defender, [LayerMask::ALL])
        ));
    }
    

    if let Some(imp) = event.impulse {
        cmd.entity(id).insert(ExternalImpulse::new(imp));
    }

    let trans = Transform::from_translation(event.pos);
    if let Some(dir) = event.dir {
        cmd.entity(id).insert(trans.looking_to(dir, Vec3::Y));
    } else {
        cmd.entity(id).insert(trans);
    }
    if let Some(lt) = event.lifetime {
        cmd.entity(id).insert((
            LifeTime(Timer::new(Duration::from_secs(lt), TimerMode::Once)),
            Released
        ));
    }
    

}

// ---

fn despawn_on_collision(
    mut collision_events: EventReader<CollisionEnded>,
    mut t_q: Query<(Entity, &mut LifeTime), With<Released>>,
) {
    for CollisionEnded(e1, e2) in collision_events.read() {
        t_q.iter_mut().for_each(|(e, mut lt)| {
            if e == *e1 || e == *e2 {
                lt.0.set_duration(Duration::ZERO); 
            }
        });
    }
}

// ---

fn despawn_on_time(
    mut t_q: Query<(Entity, &mut LifeTime)>,
    mut cmd: Commands,
    time: Res<Time>,
) {
    for (e, mut lt) in &mut t_q {
        lt.0.tick(time.delta());
        if lt.0.finished() {
            cmd.entity(e).despawn();
        }
    }
}
