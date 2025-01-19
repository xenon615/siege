#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::{
    // log::{Level, LogPlugin}, 
    prelude::*, 
    // window::{
    //     // WindowMode, 
    //     WindowPosition, 
    //     WindowResolution
    // }
};
use avian3d::{
    prelude::{
        // PhysicsDebugPlugin, 
        RigidBody
    }, 
    PhysicsPlugins
};

// use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod shared;
mod camera;
mod env;
mod trebuchet;
// mod ball;
mod field;
// mod dummies;
// mod dummy_ball;
mod fortress;
mod radar;
mod animator;
mod turret;
// mod bullet;
mod projectle;

// ---

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,
    Game
}

#[derive(Component)]
pub struct NotReady;

#[derive(Component)]
pub struct ShowGizmos;

// ---

fn main() {
    App::new()
    .insert_resource(ClearColor(Color::BLACK))
    .add_plugins((
        DefaultPlugins
        .set(
            WindowPlugin {
                primary_window : Some(Window {
                    canvas: Some("#siege-canvas".into()),
                    // resolution : WindowResolution::new(1400., 900.),
                    // mode: WindowMode::BorderlessFullscreen,
                    // position: WindowPosition::Centered(MonitorSelection::Primary),
                    ..default()
                }),
                ..default()
            },
        ),
        // .set(
        //     LogPlugin {
        //         level: Level::INFO,
        //         filter: "wgpu=error,naga=warn,bevy_gltf_components::ronstring_to_reflect_component=error,avian3d::collision::narrow_phase=error,wgpu_hal::vulkan::instance=error".to_string(),
        //         ..default()
        //     }
        // )

        PhysicsPlugins::default(),
        // PhysicsDebugPlugin::default(),
        // WorldInspectorPlugin::new(),
        camera::CameraPlugin,
        env::EnvPlugin,
        trebuchet::TrebuchetPlugin,
        
        field::FieldPlugin,
        // dummies::DummiesPlugin,
        fortress::FortressPlugin,
        radar:: RadarPlugin,
        animator::AnimatorPlugin,
        turret::TurretPlugin,
        projectle::ProjectlePlugin,
        // ball::BallPlugin,
        // bullet::BulletPlugin
        // dummy_ball::DBallPlugin,
 


    ))
    .init_state::<GameState>()
    .add_systems(Update, check_ready.run_if(in_state(GameState::Loading)))
    // .add_systems(Update, show_gizmos)
    .run();
}

// ---

fn check_ready(
    not_ready_q: Query<&NotReady>,
    mut next: ResMut<NextState<GameState>>,
) {
    if not_ready_q.is_empty() {
        info!("GAME!");
        next.set(GameState::Game);
    } 
}

// ---

#[allow(dead_code)]
fn show_gizmos ( 
    mut gismos: Gizmos,
    t_q: Query<&GlobalTransform, With<RigidBody>>
) {
    for t in t_q.iter()   {
        gismos.axes(*t, 10.);
    }
}