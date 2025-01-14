use std::f32::consts::PI;
// use avian3d::parry::na::distance_squared;
use bevy::{prelude::*, scene::SceneInstanceReady};
use crate::shared::Targetable;
use crate::{radar::RadarPositions, shared::SetTarget};
use crate::projectle::{ProjectleKey, ProjectleSpawn};
pub struct TurretPlugin;
impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, spawn.run_if(resource_added::<RadarPositions>))
        .add_systems(Update, follow.run_if(any_with_component::<Target>))
        .add_systems(Update, fire.run_if(any_with_component::<Fire>))
        .add_observer(set_target)
        .add_observer(ball_despawn)
        .add_observer(clear_target)
        ;
    }
}

// ---

#[derive(Component)]
pub struct Barrel;

#[derive(Component)]
pub struct Turret;

#[derive(Component)]
pub struct BarrelTurret(Entity);

#[derive(Component)]
pub struct Target(Entity);


#[derive(Component)]
pub struct LastShoot(f32);

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Fire;

const COOLDOWN_TIME: f32 = 0.5;
const OPERATOR_DRUNK_DEGREE: f32 = 0.2;

// ---

fn spawn (
    mut cmd: Commands,
    assets: ResMut<AssetServer>,
    rp: Res<RadarPositions>
) {
    let sh = assets.load(GltfAssetLabel::Scene(0).from_asset("models/turret.glb"));
    for p in &rp.0  {
        let pos = *p + Vec3::X * -20. * p.x.signum(); 
        cmd.spawn((
            SceneRoot(sh.clone()),
            Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(PI)),
            Turret,
        ))
        .observe(setup)
        ;
    
    }
}

// ---

fn setup (
    tr: Trigger<SceneInstanceReady>,
    mut cmd: Commands,
    props_q: Query<&GltfExtras>,
    children_q: Query<&Children>,
) {
    let turret_e = tr.entity();
    for c in children_q.iter_descendants(turret_e) {
        let Ok(props) = props_q.get(c) else {continue};

        if props.value.contains("Barrel") {
            cmd.entity(c).insert((Barrel, LastShoot(0.), BarrelTurret(turret_e)));
        }
    }
}

// ---

fn set_target(
    tr: Trigger<SetTarget>,
    target_q: Query<&Transform, Without<Turret>>,
    barrel_q: Query<(Entity, &GlobalTransform), (With<Barrel>, Without<Target>)>,
    mut cmd: Commands
) {
    let Ok(Transform{translation: target_pos , ..}) = target_q.get(tr.event().0) else {
        return;
    };
    
    let Some(barrel_e) = barrel_q.iter().min_by(|a , b| {
        a.1.translation().distance_squared(*target_pos).total_cmp(
            &b.1.translation().distance_squared(*target_pos)
        )
    }).map(|v| v.0) else {
        // println!("return");
        return;
    };
    // println!("set target pos {:?}", target_pos);
    cmd.entity(barrel_e).insert(Target(tr.event().0));
}

// --=

fn clear_target(
    tr: Trigger<OnRemove, Target>,
    mut cmd: Commands
) {
    cmd.entity(tr.entity()).remove::<Fire>();
}

// ---

fn follow(
    target_q: Query<&Transform, Without<Barrel>>,
    turret_q: Query<&GlobalTransform, (Without<Target>, Without<Barrel>)>,
    mut barrel_q: Query<(&mut Transform, &GlobalTransform, &Target, &BarrelTurret, Entity), (With<Target>, With<Barrel>)>,
    // mut gizmos: Gizmos,
    time: Res<Time>,
    mut cmd: Commands
) {
    for (mut barrel_trans, barrel_trans_g, target, barrel_turret, barrel_e) in &mut barrel_q {
        let Ok(Transform{translation: mut target_pos, ..}) = target_q.get(target.0) else {
            cmd.entity(barrel_e).remove::<Target>();
            continue;
        };

        target_pos *= fastrand::f32() * OPERATOR_DRUNK_DEGREE  + 1.; 

        if target_pos.z < barrel_trans_g.translation().z ||  target_pos.y < barrel_trans_g.translation().y {
            cmd.entity(barrel_e).remove::<Target>();
            continue;
        }
        
        let Ok(turret_trans_g) = turret_q.get(barrel_turret.0) else {
            continue;
        };

        let rotation_to_target = barrel_trans_g.compute_transform().looking_at(target_pos, Vec3::Y).rotation;
        let turret_rotation = turret_trans_g.compute_transform().rotation;
        barrel_trans.rotation = barrel_trans.rotation.slerp(
            turret_rotation.inverse().mul_quat(rotation_to_target), 
            time.delta_secs() * 50.
        );
        if barrel_trans_g.forward().dot((target_pos - barrel_trans_g.translation()).normalize()) > 0.95 {
            cmd.entity(barrel_e).insert(Fire) ;
            // let distance = target_trans.translation.distance(turret_trans_g.translation());
            // gizmos.ray(barrel_trans_g.translation(), barrel_trans_g.forward() * distance, Color::srgb(1., 1., 0.));
        } else {
            cmd.entity(barrel_e).remove::<Fire>();
            // gizmos.ray(barrel_trans_g.translation(), barrel_trans_g.forward() * 500., Color::srgb(1., 0., 1.));
        }
   }
}

// ---

fn ball_despawn(
    tr: Trigger<OnRemove, Targetable>,
    barrels_q: Query<(Entity, &Target)>,
    mut cmd: Commands
) {
    for (barrel_e, target) in &barrels_q {
        if target.0 == tr.entity() {
            cmd.entity(barrel_e).remove::<Target>();
        }
    }
}

// ---

fn fire (
    mut cmd: Commands,
    mut barrel_q: Query<(&GlobalTransform, &mut LastShoot), (With<Fire>, With<Target>)>,
    time: Res<Time>
) {

    for (gt, mut ls) in barrel_q.iter_mut() {
        let e_s = time.elapsed_secs();
        if ls.0 + COOLDOWN_TIME < e_s {
            for i in 0..12 {
                cmd.trigger(ProjectleSpawn{
                    key: ProjectleKey::Bullet,
                    pos: gt.translation() + gt.forward() * (15. + i as f32),
                    dir: None,
                    impulse: Some(gt.forward() * 1000.),
                    lifetime: Some(2)
                });
            }
            ls.0 = e_s;
        }
    }

}