use std::time::Duration;

use bevy::prelude::*;
use avian3d::prelude::*;

use crate::{GameState, NotReady};
pub struct TrebuchetPlugin;

impl Plugin for TrebuchetPlugin {
    fn build(&self, app: &mut App) {
        app
        .register_type::<Arm>()
        .register_type::<Pivot>()
        .register_type::<CounterWeight>()
        .register_type::<Bar>()

        .add_systems(Startup, startup)
        .add_systems(Update, (explore, setup).run_if(in_state(GameState::Loading)))
        .add_systems(Update, do_tension.run_if(any_with_component::<StateTension>))
        .add_systems(Update, do_arming.run_if(on_event::<CollisionEnded>()))
        .add_systems(Update, do_loose.run_if(any_with_component::<StateLoose>))
        .add_systems(Update, (reload, despawn_ball).run_if(any_with_component::<Interval>))
        .observe(enter_idle)
        .observe(enter_tension)
        ;
    }
}

// ---

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Arm;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Pivot;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct CounterWeight;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Lock;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Bar;

#[derive(Component)]
pub struct Trebuchet;

#[derive(Component)]
pub struct  SlingEnd;

#[derive(Component)]
pub struct  Ball;

#[derive(Component)]
pub struct StateIdle;

#[derive(Component)]
pub struct StateTension;

#[derive(Component)]
pub struct StateArming;

#[derive(Component)]
pub struct StateLoose;

#[derive(Component)]
pub struct Link;

#[derive(Component)]
pub struct Parts {
    pivot: Entity,
    se: Entity,
    arm: Entity,
    bar: Entity, 
    cw: Entity,
    link: Option<Entity>
}

impl Parts {
    fn is_explored(&self) -> bool {
        self.arm != Entity::PLACEHOLDER && 
        self.pivot != Entity::PLACEHOLDER && 
        self.cw != Entity::PLACEHOLDER && 
        self.bar!= Entity::PLACEHOLDER
    }

    fn new() -> Self{
        Self {pivot: Entity::PLACEHOLDER,
            se: Entity::PLACEHOLDER,
            arm: Entity::PLACEHOLDER,
            bar: Entity::PLACEHOLDER, 
            cw: Entity::PLACEHOLDER,
            link: None
        }
    }
}

#[derive(Component)]
pub struct Interval(Timer);

// -- CONSTANTS --
pub const BALL_DENSITY: f32 = 6.5;
pub const BALL_RADIUS: f32 = 0.65;

const ARM_DIM: Vec3 =  Vec3::new(1., 1., 15.);
const CW_DENSITY: f32 =  8.0;
const PIVOT_DAMPING: f32 = 0.1; 
const PIVOT_OFFSET: f32 = ARM_DIM.z  * 0.3; 
const ARM_LONG_END_Y: f32 = 3.0;

const SLING_ELEMENT_DENSITY: f32 = 100.;
const SLING_ELEMENT_COUNT: u32 = 8;
const SLING_LEN: f32 = ARM_DIM.z * 0.75;
const UNHOOKING_DOT: f32 = 0.96;

const TREBUCHET_DIM: Vec3 = Vec3::new(4., 8., 16.);  // ROUGLY


// ---

fn startup(
    mut cmd: Commands,
    assets: ResMut<AssetServer>
) {
    let asset_handle = assets.load(GltfAssetLabel::Scene(0).from_asset("models/trebuchet.glb"));
    let mut x = 0.;
    let mut z = 0.;
    for i in 0..4 {
        x += 10. * i as f32 * (if i % 2 == 0 {1.} else {-1.});
        if i % 10 == 0 {
            z += 20.;
        }
        cmd.spawn((
            SceneBundle {
                scene: asset_handle.clone(),
                transform: Transform::from_xyz(x, 0.1, z)
                ,
                ..default()
            },
            NotReady,
            Trebuchet,
            Name::new("Trebuchet"),
            RigidBody::Static,
            StateIdle,
        ));
    }
}

// ---

fn explore(
    treb_q: Query<Entity, (With<Trebuchet>, With<NotReady>, Without<Parts>)>,
    parts_q: Query<(Entity, Option<&Arm>, Option<&Pivot>, Option<&CounterWeight>, Option<&Bar>)>, 
    children: Query<&Children>,
    mut cmd: Commands,
) {
    for treb_e in treb_q.iter() {
        let mut parts = Parts::new();
        for c in children.iter_descendants(treb_e) {
            let Ok((e, oa, op, ocw, ob)) = parts_q.get(c) else {
                continue;
            };
            if oa.is_some() {
                parts.arm = e ;
            } else if op.is_some() {
                parts.pivot = e;
            } else if ob.is_some() {
                parts.bar = e;
            } else if ocw.is_some() {
                parts.cw = e;
            }
        }
        if !parts.is_explored() {
            continue;
        }
        cmd.entity(treb_e).insert(parts);
        info!("trebuchet {:?} explored ", treb_e);
    }

}


// ---

fn setup(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials : ResMut<Assets<StandardMaterial>>,
    mut treb_q: Query<(Entity, &mut Parts), (Added<Parts>,With<Trebuchet>, With<NotReady>)>,
    arm_q: Query<&Transform, With<Arm>>,
    mut cmd: Commands,
) {

    for (treb_e, mut parts) in treb_q.iter_mut() {

        // ARM ============================================================
    
        cmd.entity(parts.arm)
        .insert((
            RigidBody::Dynamic,
            // RigidBody::Static,
            MassPropertiesBundle::new_computed(&Collider::cuboid(ARM_DIM.x, ARM_DIM.y, ARM_DIM.z), 1.),
        ));
        let anchor_arm = Vec3::Z *  ARM_DIM.z * 0.5;

        // PIVOT ===========================================================
        
        cmd.entity(parts.pivot).insert(RigidBody::Static);

        let joint_id = cmd.spawn((
            RevoluteJoint::new(parts.pivot, parts.arm)
            .with_aligned_axis(Vec3::X)
            .with_local_anchor_2(-Vec3::Z * PIVOT_OFFSET)
            .with_angular_velocity_damping(PIVOT_DAMPING)
            ,
        )).id();

        cmd.entity(treb_e).add_child(joint_id);

        // CW ==============================================================

        cmd.entity(parts.cw)
        .insert((
            RigidBody::Dynamic,
            MassPropertiesBundle::new_computed(&Collider::cylinder(4., 2.), CW_DENSITY)
        ));
            
        let joint_id = cmd.spawn(
            RevoluteJoint::new(parts.arm, parts.cw)
            .with_aligned_axis(Vec3::X)
            .with_local_anchor_1(-anchor_arm)
            .with_local_anchor_2(Vec3::Y)
        ).id();

        cmd.entity(treb_e).add_child(joint_id);

        // SLING ============================================================

        let arm_pos = arm_q.get(parts.arm).unwrap().translation;
        let element_dim = Vec3::new(0.1, 0.1, SLING_LEN / SLING_ELEMENT_COUNT as f32);
        let anchor_element = Vec3::Z * element_dim.z / 2.;
        let element_mesh = meshes.add(Cuboid::from_size(element_dim));
        let element_mat = materials.add(Color::BLACK);

        let mut pos = arm_pos +  anchor_arm + anchor_element; 

        let mut prev_element_id = parts.arm;
        

        for i in 0 .. SLING_ELEMENT_COUNT {
            let element_id = cmd.spawn((
                MaterialMeshBundle {
                    transform: Transform::from_translation(pos),
                    material: element_mat.clone(),
                    mesh: element_mesh.clone(),
                    ..default()
                },
                RigidBody::Dynamic,
                MassPropertiesBundle::new_computed(
                    &Collider::cuboid(element_dim.x, element_dim.y, element_dim.z), 
                SLING_ELEMENT_DENSITY),
            )).id();
            cmd.entity(treb_e).add_child(element_id);

            let joint_id = cmd.spawn(
                SphericalJoint::new(prev_element_id, element_id)
                .with_local_anchor_1(if i == 0 {anchor_arm} else {anchor_element})
                .with_local_anchor_2(-anchor_element)
            ).id();
            
            cmd.entity(treb_e).add_child(joint_id);

            prev_element_id  = element_id;
            pos += 2. * anchor_element;
        }
        
        // ENDING =======================================================================
        
        let ending_radius = 0.1;
        let ending_id = cmd.spawn((
            MaterialMeshBundle {
                transform: Transform::from_translation(pos - anchor_element - Vec3::Z * ending_radius),
                mesh: meshes.add(Sphere::new(ending_radius)),
                material: element_mat.clone(),
                ..default()
            }, 
            RigidBody::Dynamic,
            Restitution::new(0.).with_combine_rule(CoefficientCombine::Min),
            Friction::new(0.).with_combine_rule(CoefficientCombine::Min),
            Collider::sphere(ending_radius * 5.),
            CollisionMargin(0.1),
            SlingEnd
        ))
        .id()
        ;
        cmd.entity(treb_e).add_child(ending_id);

        let joint_id = cmd.spawn(
            SphericalJoint::new(prev_element_id, ending_id)
            .with_local_anchor_1(anchor_element)
        ).id();

        cmd.entity(treb_e).add_child(joint_id);

        parts.se = ending_id;    
        cmd.entity(treb_e).remove::<NotReady>();
        info!("trebuchet {:?} setted ", treb_e );
        
    }
} 

// ---

fn enter_idle(
    trigger: Trigger<OnAdd, StateIdle>,
    mut cmd: Commands,
) {
    cmd.entity(trigger.entity()).insert(
        Interval(Timer::new(Duration::from_secs(fastrand::u64(5..10)), TimerMode::Once))
    );
}

// ---

fn enter_tension(
    trigger: Trigger<OnAdd, StateTension>,
    mut treb_q: Query<&mut Parts>,
    mut cmd: Commands,
) {
    let treb_e = trigger.entity();
    let Ok(mut parts) = treb_q.get_mut(treb_e) else {
        return;
    };
    
    let joint_id = cmd.spawn((
        DistanceJoint::new(parts.se, parts.bar)
        .with_limits(0.1, 20.)
        .with_local_anchor_2(Vec3::Z * TREBUCHET_DIM.z * 0.5 + Vec3::Y)
        ,
        Link
    )).id();
    parts.link = Some(joint_id);
    info!("trebuchet {:?} entered tension ", treb_e);
}

// ---

fn do_tension(
    treb_q: Query<(Entity, &Parts), (With<Trebuchet>, With<StateTension>)>,
    arm_q: Query<&Transform, With<Arm>>,
    mut link_q: Query<&mut DistanceJoint, With<Link>>,
    mut cmd: Commands,
) {
    for (treb_e, treb_parts)  in treb_q.iter() {
        let Ok(arm_t) = arm_q.get(treb_parts.arm) else {
            continue;
        };
      
        let arm_long_end_y = (arm_t.translation - arm_t.forward() * ARM_DIM.z * 0.5).y;
        if arm_long_end_y  < ARM_LONG_END_Y {
            cmd.entity(treb_e)
            .remove::<StateTension>()
            .insert(StateArming)
            ;
            info!("trebuchet {:?} exited tension  ", treb_e);
            continue;
        }
        let Some(link_e) = treb_parts.link else {
            continue;
        };
        let Ok(mut joint)  =  link_q.get_mut(link_e) else {
            continue;
        }; 
        let Some(limits) = joint.length_limits else {
            continue;
        };

        let j = DistanceJoint::new(joint.entity1, joint.entity2)
                .with_local_anchor_1(joint.local_anchor1);

        if limits.max > 1. {
            *joint = j
            .with_local_anchor_2(joint.local_anchor2)
            .with_limits(joint.rest_length + 1., limits.max - 0.05)
            ;
        } else {
            *joint = j
            .with_local_anchor_2(joint.local_anchor2 - Vec3::Z * 0.05)
            .with_limits(limits.min, limits.max)
            ;
    
        }
    }
}

// ---

fn do_arming(
    mut collision_events: EventReader<CollisionEnded>,
    se_q: Query<Entity, With<SlingEnd>>,
    ball_q: Query<Entity, With<Ball>>,
    parent_q: Query<&Parent>,
    parts_q: Query<&Parts>,
    mut cmd: Commands

) {
    for CollisionEnded(e1, e2)  in  collision_events.read() {
        if  se_q.contains(*e1)  &&  ball_q.contains(*e2) ||
            se_q.contains(*e2)  &&  ball_q.contains(*e1)
         {
            let (se_e, ball_e) = if se_q.contains(*e1) {(*e1, *e2)} else {(*e2, *e1)};

            for p in parent_q.iter_ancestors(se_e) {
                if let Ok(parts) =  parts_q.get(p) {

                    let Some(link_e) = parts.link else {
                        continue;
                    };

                    cmd.entity(link_e)
                    .insert(
                        DistanceJoint::new(se_e, ball_e)
                        .with_rest_length(BALL_RADIUS * 2.)
                        .with_compliance(0.001)
                        .with_linear_velocity_damping(1000.)
                    );

                    cmd.entity(p)
                    .remove::<StateArming>()
                    .insert(StateLoose);

                    cmd.entity(ball_e).insert(
                        Interval(Timer::new(Duration::from_secs(fastrand::u64(15..25)), TimerMode::Once))
                    )
                    .insert(LinearVelocity(Vec3::ZERO))
                    ;
                    info!("trebuchet {:?} armed ", p);
                }
            }
            
        }
    }    
}

// ---

fn do_loose(
    mut treb_q: Query<(Entity, &mut Parts, &Transform), (With<Trebuchet>, With<StateLoose>)>,
    mut cmd: Commands,
    se_q: Query<&GlobalTransform>,
) {

    for (treb_e, mut treb_parts, treb_t)  in treb_q.iter_mut() {
        
        let Ok(se_t) = se_q.get(treb_parts.se) else {
            continue;
        };
        let Some(link) = treb_parts.link else {
            continue;
        };

        let center = treb_t.translation  + Vec3::Y * TREBUCHET_DIM.y * 0.5;
        let to_se = (se_t.translation() - center).normalize();
        let dot = to_se.dot(Vec3::Y);
        if dot > UNHOOKING_DOT {
            cmd.entity(link).despawn();
            cmd.entity(treb_e)
            .remove::<StateLoose>()
            .insert(StateIdle)
            ;
            treb_parts.link = None;
        } 
    }
}

// ---

fn reload(
    mut t_q: Query<(Entity, &mut Interval), With<Trebuchet>>,
    mut cmd: Commands,
    time: Res<Time>
) {
    for (e, mut interval) in & mut t_q {
        interval.0.tick(time.delta());
        if interval.0.finished() {
            cmd.entity(e)
            .remove::<StateIdle>()
            .remove::<Interval>()
            .insert(StateTension)
            ;
        }
    }
}

// ---

fn despawn_ball(
    mut t_q: Query<(Entity, &mut Interval), With<Ball>>,
    mut cmd: Commands,
    time: Res<Time>
) {
    for (e, mut interval) in & mut t_q {
        interval.0.tick(time.delta());
        if interval.0.finished() {
            info!("ball despawned");
            cmd.entity(e).despawn_recursive();
        }
    }
}