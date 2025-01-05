use bevy::prelude::*;
use avian3d::prelude::*;

#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub enum GameLayer {
    Targetable,
    #[default]
    Env,
    SlingEnd
}

#[derive(Component)]
pub struct  Ball;

#[derive(Event)]
pub struct SetTarget (pub Entity);

#[derive(Component)]
pub struct  Targetable;

#[derive(Event)]
pub struct BallSpawn(pub Vec3);

#[derive(Component)]
pub struct Interval(pub Timer);

pub const BALL_DENSITY: f32 = 14.5;
pub const BALL_RADIUS: f32 = 0.55;

    


