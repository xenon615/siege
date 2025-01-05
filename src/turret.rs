use std::f32::consts::PI;
use bevy::prelude::*;
use avian3d::prelude::*;
use crate::{radar::RadarPositions, shared::{Ball, SetTarget}};

pub struct TurretPlugin;
impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app
        .register_type::<Barrel>()
        .add_systems(Update, spawn.run_if(resource_added::<RadarPositions>))
        .add_systems(Update, setup.run_if(not(any_with_component::<TurretBarrel>)))
        .add_systems(Update, follow.run_if(any_with_component::<Target>))
        .add_observer(set_target)
        .add_observer(ball_despawn)
        ;
    }
}


// ---

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Barrel;

#[derive(Component)]
pub struct Turret;

#[derive(Component)]
pub struct TurretBarrel(Entity);

#[derive(Component)]
pub struct Target(Entity);


#[derive(Component)]
pub struct LastShoot(f32);

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Fire;
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
            LastShoot(0.)
        ));
    
    }
}

// ---

fn setup (
    mut cmd: Commands,
    turret_q: Query<Entity, (With<Turret>, Without<TurretBarrel>)>,
    barrel_q: Query<&Barrel>,
    children_q: Query<&Children>,
) {
    for turret_e in &turret_q  {
        for ce in children_q.iter_descendants(turret_e) {
            if let Ok(_b) =  barrel_q.get(ce) {
                cmd.entity(turret_e).insert(TurretBarrel(ce));
            }
        }
    }
}

// ---

fn set_target(
    tr: Trigger<SetTarget>,
    turret_q: Query<Entity, (With<Turret>, Without<Target>)>,
    mut cmd: Commands
) {
    
    for e in &turret_q {
        // println!("cannon {:?} take target {:?} ", e ,tr.event().0);
        cmd.entity(e).insert(Target(tr.event().0));
        break;
    }

}

// ---

fn follow(
    target_q: Query<&Transform, Without<Barrel>>,
    turret_q: Query<(&GlobalTransform, &Target, &TurretBarrel, Entity)>,
    mut barrel_q: Query<(&mut Transform, &GlobalTransform), With<Barrel>>,
    mut gizmos: Gizmos,
    time: Res<Time>,
    mut cmd: Commands
) {
   for (turret_trans_g, turret_target, turret_barrel, turret_e) in &turret_q {
        if let Ok((mut barrel_trans, barrel_trans_g)) = barrel_q.get_mut(turret_barrel.0) {
            if let Ok(target_trans) = target_q.get(turret_target.0) {

                if target_trans.translation.z < turret_trans_g.translation().z {
                    cmd.entity(turret_e).remove::<Target>();
                }

                let rotation_to_target = barrel_trans_g.compute_transform().looking_at(target_trans.translation, Vec3::Y).rotation;
                let turret_rotation = turret_trans_g.compute_transform().rotation;
                barrel_trans.rotation = barrel_trans.rotation.slerp(
                    turret_rotation.inverse().mul_quat(rotation_to_target), 
                    time.delta_secs() * 50.
                );
                if barrel_trans_g.forward().dot((target_trans.translation - barrel_trans_g.translation()).normalize()) > 0.95 {
                    gizmos.ray(barrel_trans_g.translation(), barrel_trans_g.forward() * 100., Color::srgb(1., 1., 0.));
                } else {
                    gizmos.ray(barrel_trans_g.translation(), barrel_trans_g.forward() * 100., Color::srgb(1., 0., 1.));
                }

                
            }
        }
   }
}

// ---

fn ball_despawn(
    tr: Trigger<OnRemove, Ball>,
    turrets_q: Query<(Entity, &Target), With<Turret>>,
    mut cmd: Commands
) {
    for (turret_e, turret_target) in &turrets_q {
        if turret_target.0 == tr.entity() {
            cmd.entity(turret_e).remove::<Target>();
        }
    }
}
