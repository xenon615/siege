use bevy::{
    prelude::*,
    scene::SceneInstanceReady
};
use avian3d:: prelude::*;
use crate::{field::FortressPosition, NotReady};


pub struct FortressPlugin;
impl Plugin for FortressPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, startup.run_if(resource_added::<FortressPosition>))
        ;
    }
}

// ---

#[derive(Component)]
pub struct FortressTMP;


#[derive(Component)]
pub struct Fortress;

fn startup(
    mut cmd: Commands,
    assets: ResMut<AssetServer>,
    fp: Res<FortressPosition>
) {
    cmd.spawn((NotReady, FortressTMP, Name::new("Fortress")));
    cmd.spawn((
        SceneRoot(assets.load(GltfAssetLabel::Scene(0).from_asset("models/fortress.glb"))),
        Transform::from_translation(fp.0),   
        Fortress,
        Name::new("Fortress"),
    ))
    .observe(setup)
    ;
}

// ---

fn setup (
    tr: Trigger<SceneInstanceReady>,
    mut cmd: Commands,
    children_q: Query<&Children>,
    name_q: Query<&Name>,
    ready_q: Single<Entity, (With<FortressTMP>, With<NotReady>)>,

) {
    for  c in children_q.iter_descendants_depth_first(tr.entity()) {
        if let Ok(name) = name_q.get(c) {
            if name.starts_with("brick") || name.starts_with("disk") || name.starts_with("pillar") || name.starts_with("roof") {
                cmd.entity(c).insert((
                    // RigidBody::Dynamic,
                    RigidBody::Static,
                    ColliderDensity(0.1),
                    Friction::new(0.1)
                ));

                if name.starts_with("brick") {
                    cmd.entity(c).insert(Collider::cuboid(8., 4., 4.));
                } else if name.starts_with("pillar") {
                    cmd.entity(c).insert(Collider::cylinder(3., 8.));
                } else if name.starts_with("roof") {
                    cmd.entity(c).insert(Collider::cone(10., 15.));
                } else if name.starts_with("disk") {
                    cmd.entity(c).insert(Collider::cylinder(10., 2.));
                } 
            }
        }
    }
    cmd.entity(ready_q.into_inner()).despawn();    
}