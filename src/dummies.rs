use std::f32::consts::PI;

use bevy::prelude::*;
use avian3d::prelude::*;
use crate::field::FieldTarget;


use crate::{GameState, NotReady};
pub struct DummiesPlugin;
impl Plugin for DummiesPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, startup)
        .add_systems(Update, setup.run_if(in_state(GameState::Loading)))
        ;

    }
}

// ---
#[derive(Component)]
pub struct Dummy;

// ---

fn startup(mut cmd: Commands) {
    cmd.spawn((NotReady, Dummy));
}

// ---

fn setup(
    mut cmd: Commands,
    tf_q: Query<&Transform, With<FieldTarget>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    ready_q: Query<Entity, (With<Dummy>, With<NotReady>)>
) {
    if ready_q.is_empty() {
        return;
    }

    let Ok(t) = tf_q.get_single() else {
        return;
    };

    if let Ok(re) = ready_q.get_single() {
        cmd.entity(re).despawn();    
    }

    // let materials_list: Vec<Handle<StandardMaterial>> = (0..10).map(|i| {
    //     let h = i as f32 * 30.; 
    //     materials.add(Color::hsl(h, 1., 0.5))
    // }).collect();

    let material = materials.add(Color::hsl(100., 1., 0.2));

    let dummy_dim = Vec3::new(4., 4., 4.);
    let mesh_h = meshes.add(Cuboid::from_size(dummy_dim));
    
    // for pos in spiral(t.translation + Vec3::new(20., 0., 20.), dummy_dim, 100) {
    // for (i, pos) in 
    for pos in 
    wall(t.translation + Vec3::new(-44., 0., -40.), dummy_dim, 110)
    // .chain(spiral(t.translation + Vec3::new(0., 0., 0.), dummy_dim, 100))
    // .enumerate() 
    {    
        // let color_index = i / 110;
        cmd.spawn((
            PbrBundle {
                mesh: mesh_h.clone(),
                // material: materials_list[color_index].clone(),
                material: material.clone(),
                transform: Transform::from_translation(pos),
                ..default()
            },
            RigidBody::Dynamic,
            Restitution::new(0.1).with_combine_rule(CoefficientCombine::Min),
            Collider::cuboid(dummy_dim.x, dummy_dim.y, dummy_dim.z),
            ColliderDensity(0.1),
            Dummy
        ));
    }
}

// ---

#[allow(dead_code)]
fn spiral(start: Vec3, dummy_dim: Vec3, dummies_count: u32) -> impl Iterator<Item = Vec3> {
    let mut pos = start.with_y(dummy_dim.y * 0.55); 
    let step = Vec3::Z * 1.1 * dummy_dim.z.max(dummy_dim.x) ;
    let mut angle = 0.;
    let mut in_row = 0;
    let mut row_cap = 2;

    (0 .. dummies_count).map(move |_| {
        if in_row == row_cap {
            angle += PI * 0.5;
            row_cap += 1; 
            in_row = 0;
        } 
        in_row += 1;
        
        pos += Quat::from_rotation_y(angle).mul_vec3(step);
        pos
    })
} 

// ---

fn wall(start: Vec3, dummy_dim: Vec3, dummies_count: u32) -> impl Iterator<Item = Vec3> {
    let mut pos = start.with_y(dummy_dim.y * 0.55); 
    let mut step = Vec3::X * 1.1 * dummy_dim.z.max(dummy_dim.x) ;
    let mut in_row = 0;
    let mut row_cap = 20;
    (0 .. dummies_count).map(move |_| {
        if in_row == row_cap {
            row_cap -= 2; 
            in_row = 0;
            step *= -1.;
            pos.y += 1.1 * dummy_dim.y;
        } 
        in_row += 1;
        
        pos += step;
        pos
    })
} 

