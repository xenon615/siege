use bevy::prelude::*;
use avian3d::prelude::*;

use crate::{
    trebuchet::{
        Trebuchet,StateArming, Ball, BALL_DENSITY, BALL_RADIUS
    }, GameState, NotReady};
pub struct TrebuchetLoaderPlugin;
impl Plugin for TrebuchetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, spawn)
        .add_systems(Update, setup.run_if(in_state(GameState::Loading)))
        .observe(enter_arming)
        ;
    }
}

// ---

#[derive(Component)]
pub struct TrebuchetLoader;

#[derive(Resource)]
pub struct  TrebuchetLoaderScene(Handle<Scene>);

#[derive(Component)]
pub struct Serviced(Entity);


// ---

fn spawn(
    assets: ResMut<AssetServer>,
    mut cmd: Commands,
) {
    cmd.insert_resource(
        TrebuchetLoaderScene(assets.load(GltfAssetLabel::Scene(0).from_asset("models/trebuchet_loader.glb")))
    );
    cmd.spawn((TrebuchetLoader, NotReady));
}

// ---

fn setup(
    mut cmd: Commands,
    treb_q: Query<(Entity, &Transform), (With<Trebuchet>, Without<NotReady>)>,
    loader_res: Res<TrebuchetLoaderScene>,
    assets_gltf: Res<Assets<Scene>>,
    check_q: Query<Entity, (With<NotReady>, With<TrebuchetLoader>)>,
    mut test: Local<bool>
) {
    if assets_gltf.get(&loader_res.0).is_none() {
        return;
    }
    if treb_q.is_empty() && !*test {
        *test = true;
    } 

    if treb_q.is_empty() && *test {
        if let Ok(ready_e) = check_q.get_single() {
            cmd.entity(ready_e).despawn();
        }
    } 

    for (e, t)  in &treb_q {
        cmd.spawn((
            SceneBundle {
                scene: loader_res.0.clone(),
                transform: Transform::from_translation(t.translation - Vec3::Z * 10.),
                ..default()    
            },
            TrebuchetLoader,
            Name::new("TrebuchetLoader"),
            RigidBody::Static,
            Serviced(e),
        ));        
    }
}

// ---

fn enter_arming(
    trigger: Trigger<OnAdd, StateArming>,
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    treb_loader_q: Query<(&Transform, &Serviced), With<TrebuchetLoader>>,
) {
    let treb_e = trigger.entity();
    let Some((loader_t, _)) = treb_loader_q.iter().find(|(_, s)| {
        s.0 == treb_e
    }) else {
        return;
    };

    
    cmd.spawn((
        MaterialMeshBundle {
            transform: Transform::from_translation(loader_t.translation.with_y(5.) - Vec3::Z * 2.),
            mesh: meshes.add(Sphere::new(BALL_RADIUS)),
            material: materials.add(Color::hsl(150., 1.0, 0.5)),
            ..default()
        }, 
        RigidBody::Dynamic,
        Restitution::new(0.).with_combine_rule(CoefficientCombine::Min),
        Collider::sphere(BALL_RADIUS),
        ColliderDensity(BALL_DENSITY),
        Ball
    ))
    ;
    
    info!("trebuchet {:?} entered arming ", treb_e);
}



