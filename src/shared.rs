use bevy::prelude::*;
use avian3d::prelude::*;

#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub enum GameLayer {
    Attacker,
    Defender,
    #[default]
    Env,
}


#[derive(Event)]
pub struct SetTarget (pub Entity);

#[derive(Component)]
pub struct  Targetable;

#[derive(Component)]
pub struct Interval(pub Timer);


    


