use bevy::prelude::*;
use avian3d::prelude::{Collider, ColliderConstructor, ColliderConstructorHierarchy, Joint, RevoluteJoint, RigidBody};

use crate::{GameState, NotReady};
pub struct TrebuchetPlugin;

impl Plugin for TrebuchetPlugin {
    fn build(&self, app: &mut App) {
        app
        .register_type::<CounterWeight>()
        .register_type::<Arm>()
        .register_type::<Pivot>()

        .add_systems(Startup, startup)
        .add_systems(Update, setup.run_if(in_state(GameState::Loading)))

        ;
    }
}

// ---

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct CounterWeight;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Arm;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Pivot;

#[derive(Component)]
pub struct Trebuchet;



// ---


fn startup(
    mut cmd: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    assets: ResMut<AssetServer>

) {
    // cmd.spawn((
    //     PbrBundle {
    //         material: materials.add(Color::BLACK),
    //         mesh: meshes.add(Cuboid::from_length(5.)),
    //         transform: Transform::from_xyz(0., 5.0, 0.),
    //         ..default()
    //     },
    //     RigidBody::Dynamic,
    //     Collider::cuboid(5., 5., 5.)
    // ));

    cmd.spawn((
        SceneBundle {
            scene: assets.load(GltfAssetLabel::Scene(0).from_asset("models/trebuchet.glb")),
            ..default()
        },
        NotReady,
        Trebuchet,
        Name::new("Trebuchet"),
        ColliderConstructorHierarchy::new(None)
        .with_constructor_for_name("m_stand_l", ColliderConstructor::TrimeshFromMesh)
        .with_constructor_for_name("m_stand_r", ColliderConstructor::TrimeshFromMesh)
        .with_constructor_for_name("m_arm", ColliderConstructor::TrimeshFromMesh)
        .with_constructor_for_name("m_counterweight", ColliderConstructor::TrimeshFromMesh),
    ));
}

// ---

fn setup(
    trebuchet_q: Query<Entity, (With<Trebuchet>, With<NotReady>)>,
    arm_q: Query<Entity, With<Arm>>,
    pivot_q: Query<Entity, With<Pivot>>,
    cw_q: Query<Entity, With<CounterWeight>>,
    mut cmd: Commands,
    children: Query<&Children>
) {

    for parent_e in trebuchet_q.iter() {
        let mut e_arm = Entity::PLACEHOLDER;
        let mut e_pivot = Entity::PLACEHOLDER;
        let mut e_cw = Entity::PLACEHOLDER;

        for c in children.iter_descendants(parent_e) {
            if let Ok(arm) =  arm_q.get(c) {
                e_arm = arm
            }
            if let Ok(pivot) =  pivot_q.get(c) {
                e_pivot = pivot
            }
            if let Ok(cw) =  cw_q.get(c) {
                e_cw = cw
            }

        }
        if e_arm != Entity::PLACEHOLDER && e_pivot != Entity::PLACEHOLDER  && e_cw != Entity::PLACEHOLDER{

            // let joint_id = cmd.spawn(
            //     RevoluteJoint::new(e_pivot, e_arm)
            //     .with_aligned_axis(Vec3::X)
            // ).id();
            // cmd.entity(parent_e).add_child(joint_id);
            cmd.entity(e_arm).insert(RigidBody::Static);
            cmd.entity(e_cw).insert(RigidBody::Dynamic);
            let joint_id = cmd.spawn(
                RevoluteJoint::new(e_arm, e_cw)
                .with_aligned_axis(Vec3::X)
                .with_local_anchor_1(Vec3::new(0., 0., -3.))
                // .with_local_anchor_2(Vec3::Y * 1.0)
            ).id();
            cmd.entity(parent_e).add_child(joint_id);



            cmd.entity(parent_e).remove::<NotReady>();
        }

    }
} 