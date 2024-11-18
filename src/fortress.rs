use avian3d::parry::utils::hashmap::HashMap;
use bevy::prelude::*;
use avian3d::prelude::*;

use crate::field::FieldTarget;
use crate::{GameState, NotReady};


pub struct FortressPlugin;
impl Plugin for FortressPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, startup)
        .add_systems(Update, setup.run_if(in_state(GameState::Loading)))
        ;
    }
}

// ---
#[derive(Component)]
pub struct FortressTMP;


#[derive(Component)]
pub struct Fortress;

#[derive(Component)]
pub struct PartOk;


// pub  enum  BuildingPartsKey {
//     Brick,
//     Pillar,
//     Platform,
//     Roof
// }

// #[derive(Resource)]
// pub struct BuildingParts(HashMap<BuildingPartsKey, Handle<Mesh>>);

// // ---

// fn startup(mut cmd: Commands) {
//     cmd.spawn((NotReady, Buildings));
// }

// // ---

// fn setup(
//     mut cmd: Commands,
//     tf_q: Query<&Transform, With<FieldTarget>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     ready_q: Query<Entity, (With<Buildings>, With<NotReady>)>

// ) {

//     if ready_q.is_empty() {
//         return;
//     }

//     let Ok(t) = tf_q.get_single() else {
//         return;
//     };

//     if let Ok(re) = ready_q.get_single() {
//         cmd.entity(re).despawn();    
//     }


// }

fn startup(
    mut cmd: Commands,
    assets: ResMut<AssetServer>
) {
    cmd.spawn((NotReady, FortressTMP));
    cmd.spawn((
        SceneBundle {
            transform: Transform::from_xyz(0., 0., -300.),
            scene: assets.load(GltfAssetLabel::Scene(0).from_asset("models/fortress.glb")),
            ..default()
        },
        Fortress,
        Name::new("Fortress"),
    ));
}

// ---

fn setup (
    mut cmd: Commands,
    children_q: Query<&Children>,
    rb_q: Query<Entity, (With<RigidBody>, Without<PartOk>)>,
    f_q : Query<Entity, With<Fortress>>,
    ready_q: Query<Entity, (With<FortressTMP>, With<NotReady>)>,
    mut first: Local<bool>
) {
    if ready_q.is_empty() {
        return;
    }

    let Ok(f_e) = f_q.get_single() else {
        return;
    };
    let mut found = false; 
    for  c in children_q.iter_descendants(f_e) {
        if let Ok(e) = rb_q.get(c) {
            found = true;
            *first = true;
            cmd.entity(e).insert(ColliderDensity(0.2));
            cmd.entity(e).insert(PartOk);
        }
    }

    if !found && *first{
        if let Ok(re) = ready_q.get_single() {
            cmd.entity(re).despawn();    
        }
    }
}