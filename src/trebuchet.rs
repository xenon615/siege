use bevy::{gizmos::gizmos, input::keyboard::KeyboardInput, math::VectorSpace, prelude::*};
use avian3d::{math::PI, prelude::*};

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
        .add_systems(Update, setup.run_if(in_state(GameState::Loading)))
        .add_systems(Update, key_control.run_if(on_event::<KeyboardInput>()))
        .add_systems(Update, do_tension.run_if(any_with_component::<State_Tension>))
        .add_systems(Update, do_arming.run_if(on_event::<CollisionEnded>()))
        .add_systems(Update, do_loose.run_if(any_with_component::<State_Loose>))

        .observe(enter_tension)
        .observe(exit_tension)
        .observe(enter_arming)
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
pub struct  PivotJoint;

#[derive(Component)]
pub struct State_Idle;

#[derive(Component)]
pub struct State_Tension;

#[derive(Component)]
pub struct State_Arming;

#[derive(Component)]
pub struct State_Loose;

#[derive(Component)]
pub struct Focused;

#[derive(Component)]
pub struct Link;

#[derive(Component)]
pub struct Parts {
    pivot: Entity,
    se: Entity,
    arm: Entity,
    bar: Entity, 
    link: Option<Entity>
}

const ARM_DIM: Vec3 =  Vec3::new(1., 1., 15.);

// ---

fn startup(
    mut cmd: Commands,
    assets: ResMut<AssetServer>

) {
    cmd.spawn((
        SceneBundle {
            scene: assets.load(GltfAssetLabel::Scene(0).from_asset("models/trebuchet.glb")),
            transform: Transform::from_xyz(0., 0., 0.)
            .with_rotation(Quat::from_rotation_y(PI))
            ,
            ..default()
        },
        NotReady,
        Trebuchet,
        Name::new("Trebuchet"),
        RigidBody::Static,
        Focused,
        // ColliderConstructorHierarchy::new(None)
        // .with_constructor_for_name("m_bar_slope", ColliderConstructor::TrimeshFromMesh),
        State_Idle,
    ));
}

// ---

fn setup(
    trebuchet_q: Query<Entity, (With<Trebuchet>, With<NotReady>)>,
    arm_q: Query<(Entity, &Transform), With<Arm>>,
    pivot_q: Query<Entity, With<Pivot>>,
    cw_q: Query<Entity, With<CounterWeight>>,
    bar_q: Query<Entity, With<Bar>>,
    mut cmd: Commands,
    children: Query<&Children>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials : ResMut<Assets<StandardMaterial>>,

) {

    for parent_e in trebuchet_q.iter() {
        let mut e_arm = Entity::PLACEHOLDER;
        let mut e_pivot = Entity::PLACEHOLDER;
        let mut e_cw = Entity::PLACEHOLDER;
        let mut e_bar = Entity::PLACEHOLDER;
        let mut arm_pos = Vec3::ZERO;

        for c in children.iter_descendants(parent_e) {
            if let Ok((arm, arm_t)) =  arm_q.get(c) {
                e_arm = arm;
                arm_pos = arm_t.translation;
            }
            if let Ok(pivot) =  pivot_q.get(c) {
                e_pivot = pivot
            }

            if let Ok(bar) =  bar_q.get(c) {
                e_bar = bar
            }

            if let Ok(cw) =  cw_q.get(c) {
                e_cw = cw
            }

        }
        if e_arm == Entity::PLACEHOLDER || e_pivot == Entity::PLACEHOLDER  || e_cw == Entity::PLACEHOLDER {
            continue;
        }

        // ARM ============================================================
    
        cmd.entity(e_arm)
        .insert((
            RigidBody::Dynamic,
            // RigidBody::Static,
            MassPropertiesBundle::new_computed(&Collider::cuboid(ARM_DIM.x, ARM_DIM.y, ARM_DIM.z), 1.),
        ));
        let anchor_arm = Vec3::Z *  ARM_DIM.z * 0.5;

        // PIVOT ===========================================================

        let pivot_offset = ARM_DIM.z  * 0.3;
        cmd.entity(e_pivot).insert(RigidBody::Static);

        let joint_id = cmd.spawn((
            RevoluteJoint::new(e_pivot, e_arm)
            .with_aligned_axis(Vec3::X)
            .with_local_anchor_2(-Vec3::Z * pivot_offset)
            .with_angular_velocity_damping(10.)
            ,
            PivotJoint
        )).id();

        cmd.entity(parent_e).add_child(joint_id);

        // CW ==============================================================

        cmd.entity(e_cw)
        .insert((
            RigidBody::Dynamic,
            // RigidBody::Static,
            MassPropertiesBundle::new_computed(&Collider::cylinder(4., 2.), 10.)
        ));
            
        let joint_id = cmd.spawn(
            RevoluteJoint::new(e_arm, e_cw)
            .with_aligned_axis(Vec3::X)
            .with_local_anchor_1(-anchor_arm)
            .with_local_anchor_2(Vec3::Y * 1.5)
        ).id();

        cmd.entity(parent_e).add_child(joint_id);

        // SLING ============================================================
        
        let sling_len = ARM_DIM.z * 0.8;
        let element_count = 4;
        let element_dim = Vec3::new(0.1, 0.1, sling_len / element_count as f32);
        let anchor_element = Vec3::Z * element_dim.z / 2.;
        let element_mesh = meshes.add(Cuboid::from_size(element_dim));
        let element_mat = materials.add(Color::BLACK);

        let mut pos = arm_pos +  anchor_arm + anchor_element; 

        let mut prev_element_id = e_arm;
        

        for i in 0 .. element_count {
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
                50.),
            )).id();
            cmd.entity(parent_e).add_child(element_id);

            let joint_id = cmd.spawn(
                SphericalJoint::new(prev_element_id, element_id)
                .with_local_anchor_1(if i == 0 {anchor_arm} else {anchor_element})
                .with_local_anchor_2(-anchor_element)
            ).id();
            
            cmd.entity(parent_e).add_child(joint_id);

            prev_element_id  = element_id;
            pos += 2. * anchor_element;
        }
        
        // ENDING =======================================================================
        
        let ending_radius = 0.2;
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
            Collider::sphere(ending_radius),
            CollisionMargin(0.1),
            SlingEnd
        ))
        .id()
        ;
        cmd.entity(parent_e).add_child(ending_id);

        // let joint_id = cmd.spawn((
        //     RevoluteJoint::new(prev_element_id, ending_id)
        //     .with_aligned_axis(Vec3::X)
        //     .with_local_anchor_1(anchor_element)
        //     .with_compliance(0.),
        // )).id();

        let joint_id = cmd.spawn((
            RevoluteJoint::new(prev_element_id, ending_id)
            .with_local_anchor_1(anchor_element)
            .with_compliance(0.),
        )).id();

        cmd.entity(parent_e).add_child(joint_id);

    // ---

        cmd.entity(parent_e).remove::<NotReady>();
        
        cmd.entity(parent_e).insert(Parts{
            se: ending_id,
            pivot: e_pivot,
            arm: e_arm,
            bar: e_bar,
            link: None            
        });
    }
} 

// ---

fn key_control(
    keys: Res<ButtonInput<KeyCode>>,
    t_q: Query<Entity, (With<Trebuchet>, With<Focused>)>,
    mut cmd: Commands
) {

    let Ok(t_e) =  t_q.get_single() else {
        return;
    };

    if keys.just_pressed(KeyCode::ArrowDown) {
        cmd.entity(t_e)
        .remove::<State_Idle>()
        .insert(State_Tension);
    }

}

// ---

fn enter_tension(
    trigger: Trigger<OnAdd, State_Tension>,
    mut treb_q: Query<&mut Parts>,
    mut pivot_q: Query<&mut RevoluteJoint>,
    mut cmd: Commands,
) {
    let Ok(mut parts) = treb_q.get_mut (trigger.entity()) else {
        return;
    };
    if let Ok(mut rj) = pivot_q.get_mut(parts.pivot) {
        rj.damping_angular = 1000.0;
    }
    println!("enter_tension");
    let joint_id = cmd.spawn((
        DistanceJoint::new(parts.se, parts.bar)
        .with_limits(0.1, 20.)
        .with_local_anchor_2(Vec3::Z * 8.  + Vec3::Y)
        ,
        Link
    )).id();
    parts.link = Some(joint_id);
}

// ---

fn do_tension(
    treb_q: Query<(Entity, &Parts), With<Trebuchet>>,
    arm_q: Query<&Transform, With<Arm>>,
    mut link_q: Query<&mut DistanceJoint, With<Link>>,
    mut cmd: Commands,
) {
    for (treb_ent, treb_parts)  in treb_q.iter() {
        let Ok(arm_t) = arm_q.get(treb_parts.arm) else {
            continue;
        };

        let arm_long_end_y = (arm_t.translation - arm_t.forward() * ARM_DIM.z * 0.5).y;
        if arm_long_end_y -1. < 0. {
            cmd.entity(treb_ent)
            .remove::<State_Tension>()
            .insert(State_Arming)
            ;
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
            .with_limits(limits.min, limits.max - 0.05)
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

fn exit_tension(
    trigger: Trigger<OnRemove, State_Tension>,
    treb_q: Query<&Parts>,
    mut pivot_q: Query<&mut RevoluteJoint>,
) {
    println!("exit tension");
    let Ok(parts) = treb_q.get(trigger.entity()) else {
        return;
    };
    if let Ok(mut rj) = pivot_q.get_mut(parts.pivot) {
        rj.damping_angular = 0.1;
    }
}

// ----

fn enter_arming(
    trigger: Trigger<OnAdd, State_Arming>,
    treb_q: Query<&Transform>,
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ball_radius = 0.5;

    let Ok(trans) = treb_q.get(trigger.entity()) else {
        return;
    };

    cmd.spawn((
        MaterialMeshBundle {
            transform: Transform::from_translation(trans.translation.with_y(5.) - Vec3::Z * 5.),
            mesh: meshes.add(Sphere::new(ball_radius)),
            material: materials.add(Color::WHITE),
            ..default()
        }, 
        RigidBody::Dynamic,
        Restitution::new(0.).with_combine_rule(CoefficientCombine::Min),
        Friction::new(0.).with_combine_rule(CoefficientCombine::Min),
        Collider::sphere(ball_radius),
        ExternalForce::new(Vec3::Z * 0.5),
        Ball
    ));
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
            let se_e = if se_q.contains(*e1) {e1} else {e2};
            for p in parent_q.iter_ancestors(*se_e) {
                if let Ok(parts) =  parts_q.get(p) {

                    let Some(link_e) = parts.link else {
                        continue;
                    };

                    cmd.entity(link_e)
                    .remove::<DistanceJoint>()
                    .insert(FixedJoint::new(*e1, *e2));

                    cmd.entity(p)
                    .remove::<State_Arming>()
                    .insert(State_Loose);
                }
            }
            println!("booch!");
        }
    }    
}

// ---

fn do_loose(
    mut treb_q: Query<(Entity, &mut Parts, &Transform), With<Trebuchet>>,
    mut cmd: Commands,
    se_q: Query<&GlobalTransform>,
) {

    for (treb_ent, mut treb_parts, treb_t)  in treb_q.iter_mut() {
        
        let Ok(se_t) = se_q.get(treb_parts.se) else {
            continue;
        };

        let center = treb_t.translation  + Vec3::Y * 5.;
        let to_se = (se_t.translation() - center).normalize();
        let dot = to_se.dot(Vec3::Y);
        if dot > 0.95 {
            cmd.entity(treb_parts.link.unwrap()).despawn();
            cmd.entity(treb_ent)
            .remove::<State_Loose>()
            .insert(State_Idle);
            treb_parts.link = None;
        } 
    }
}