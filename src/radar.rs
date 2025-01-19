use std::f32::consts::PI;
use avian3d::prelude::{Collider, ShapeCastConfig, SpatialQuery, SpatialQueryFilter};
use bevy::scene::SceneInstanceReady;
use bevy::{
    // gizmos::gizmos,
    prelude::*};
use crate::{animator::*, GameState};
use crate::shared::{GameLayer, SetTarget, Targetable};
use crate::field::FortressPosition;


pub struct RadarPlugin;
impl Plugin for RadarPlugin {
    fn build(&self, app: &mut App) {
        app
        // .register_type::<Antenna>()
        .add_systems(Update, startup.run_if(resource_added::<FortressPosition>))
        .add_systems(OnEnter(GameState::Game), enter_game)
        .add_systems(Update, scan.run_if(in_state(GameState::Game)))
        .add_observer(targetable_despawn)

        ;
    }
}

// ---

#[derive(Component)]
pub struct Radar;

#[derive(Resource)]
pub struct RadarPositions(pub Vec<Vec3>);

#[derive(Resource)]
pub struct RadarTargets(pub Vec<Entity>);

#[derive(Component)]
pub struct Antenna;

// ---

fn startup(
    mut cmd: Commands,
    assets: ResMut<AssetServer>,
    mut all_animations: ResMut<AllAnimations>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    ftp: Res<FortressPosition>
) {
    all_animations.add(AnimationKey::Radar, "models/radar.glb", 2, &mut graphs, &assets);
    let sh = assets.load(GltfAssetLabel::Scene(0).from_asset("models/radar.glb"));
    let mut rp = RadarPositions(Vec::new());

    for i in 0..2 {
        let pos = ftp.0 + Vec3::X * (if i == 0 {-1.} else {1.} * 100.) + Vec3::Z * 100.; 
        cmd.spawn((
            SceneRoot(sh.clone()),
            Transform::from_translation(pos.with_y(0.)).with_rotation(Quat::from_rotation_y(PI)),
            Radar,
            AnimationKey::Radar,
            Name::new("Radar")
        ))
        .observe(setup)
        ;
        rp.0.push(pos);
    }
    cmd.insert_resource(rp);
    cmd.insert_resource(RadarTargets(Vec::new()));
}

// ---

fn setup(
    tr: Trigger<SceneInstanceReady>,
    children: Query<&Children>,
    extras: Query<&GltfExtras>,
    mut cmd: Commands
) {
    for c in children.iter_descendants_depth_first(tr.entity()) {
        if let Ok(props) = extras.get(c) {
            if props.value.contains("Antenna") {
                cmd.entity(c).insert(Antenna);
            }
        }
    } 
}

// ---

fn enter_game(
    mut ca_q: Query<&mut CurrentAnimation, With<Radar>>
) {
    for mut ca in ca_q.iter_mut() {
        ca.0 = 1;
    }
}

// ---

fn scan(
    ant_q: Query<&GlobalTransform, With<Antenna>>,
    spatial: SpatialQuery,
    // mut gizmos: Gizmos,
    mut cmd: Commands,
    mut targets: ResMut<RadarTargets>,
) {
    let collider_dim = Vec3::new(50., 200., 50.);
    for t  in &ant_q {
        // gizmos.ray(t.translation()  +  t.forward() * 5., t.forward() * 100., Color::hsl(10., 1., 0.5));

        if let Some(shd) = spatial.cast_shape(
            &Collider::cuboid(collider_dim.x, collider_dim.y, collider_dim.z), 
            t.translation() + t.forward() * 15., 
            Quat::IDENTITY, 
            t.forward(), 
            &ShapeCastConfig {
                max_distance: 160.,
                ignore_origin_penetration: true,
                ..default()
            },
            &SpatialQueryFilter::from_mask(GameLayer::Attacker)
        ) {
            if targets.0.iter().find(|el| {
                **el == shd.entity
            }).is_none() {
                cmd.trigger(SetTarget(shd.entity));
                targets.0.push(shd.entity);
            }
        }
    }
}

// ---

fn targetable_despawn(
    tr: Trigger<OnRemove, Targetable>,
    mut targets: ResMut<RadarTargets>,
) {
    if let Some(idx) = targets.0.iter().position(|e| {
        *e == tr.entity()
    }) {
        targets.0.swap_remove(idx);
    }
}

