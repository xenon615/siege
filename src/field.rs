use avian3d::prelude::*;
use bevy::{prelude::*, scene::SceneInstanceReady};

use crate::shared::GameLayer;
pub struct FieldPlugin;
impl Plugin for FieldPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, startup)
        ;
    }
}

// ---

#[derive(Resource, Default)]
pub struct FortressPosition(pub Vec3);

// ---

fn startup(
    mut cmd: Commands,
    assets: ResMut<AssetServer>
) {
    cmd.spawn((
        SceneRoot(assets.load(GltfAssetLabel::Scene(0).from_asset("models/field.glb"))),
        ColliderConstructorHierarchy::new(None).with_constructor_for_name("m_target", ColliderConstructor::TrimeshFromMesh),
        CollisionLayers::new([GameLayer::Env], [LayerMask::ALL])
     ))
     .observe(setup)
     ;
}

// ---

fn setup(
    tr: Trigger<SceneInstanceReady>,
    mut cmd: Commands,
    children: Query<&Children>,
    props: Query<(&GltfExtras, &Transform)>
) {
    for c in children.iter_descendants_depth_first(tr.entity()) {
        let Ok((ex, t)) = props.get(c) else {
            continue;
        };
        if ex.value.contains("FieldTarget") {
            cmd.insert_resource(FortressPosition(t.translation));
            cmd.entity(c).insert(RigidBody::Static);
        }
    }
}
