use bevy::prelude::*;
use bevy::utils::HashMap;
use std::time::Duration;
use crate::{GameState, NotReady};

// ---

#[derive(Component)]
pub struct  CurrentAnimation(pub usize, Entity);

pub struct AnimationSet {
    pub animations: Vec<AnimationNodeIndex>,
    pub graph: Handle<AnimationGraph>,
}

#[derive(Component, PartialEq, Eq, Hash, Debug)]
pub enum AnimationKey {
    Radar,
}

#[derive(Resource)]
pub struct AllAnimations(pub HashMap<AnimationKey, AnimationSet>);
impl AllAnimations {
    pub fn add(&mut self, key: AnimationKey, path: &'static str, count: usize, graphs: &mut ResMut<Assets<AnimationGraph>>, assets: &ResMut<AssetServer>) {
        let mut graph = AnimationGraph::new();
        self.0.insert(
            key, 
            AnimationSet {
                animations: graph
                    .add_clips((0..count).map(|i| {assets.load(GltfAssetLabel::Animation(i).from_asset(path))}), 1.0,graph.root)
                    .collect(),
                graph: graphs.add(graph),
            }
        );
    }
}

#[derive(Component)]
pub struct TempAnimatorMarker;

// ---

pub struct AnimatorPlugin;
impl Plugin for AnimatorPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, startup)
        .add_systems(Update, setup.run_if(in_state(GameState::Loading)))
        .add_systems(Update, switch)
        .insert_resource(AllAnimations(HashMap::new()))
        .add_observer(check)
        ;
    }
}

// ---

fn startup(
    mut cmd: Commands
) {
    cmd.spawn((NotReady, TempAnimatorMarker));
}

// ---

pub fn setup(
    mut cmd: Commands,
    all_animations: Res<AllAnimations>,
    mut players: Query<&mut AnimationPlayer>,
    objects_q: Query<(Entity, &AnimationKey), (Without<CurrentAnimation>, With<AnimationKey>)>, 
    children_q : Query<&Children>
) {
    if objects_q.is_empty() {
        return;
    }

    for (o_entity, o_akey) in objects_q.iter() {
        for c  in children_q.iter_descendants(o_entity) {
            if let Ok(mut player) = players.get_mut(c)  {
                let Some(ani_set) =  all_animations.0.get(o_akey) else {
                    return;
                };
                let last_animation = ani_set.animations.len() - 1;
                let mut transitions = AnimationTransitions::new();
                transitions
                    .play(&mut player, ani_set.animations[last_animation] , Duration::ZERO)
                    .repeat()
                ;
                cmd
                    .entity(c)
                    .insert(AnimationGraphHandle(ani_set.graph.clone()))
                    .insert(transitions)
                ;
                cmd.entity(o_entity).insert(CurrentAnimation(last_animation, c));
            }
        }
    }
    
}

// ---

pub fn switch(
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    objects_q: Query<(&CurrentAnimation, &AnimationKey), Changed<CurrentAnimation>>,
    all_animations: Res<AllAnimations>,
) {
    for (ca, akey) in objects_q.iter() {
        if let Ok((mut player, mut transitions)) = animation_players.get_mut(ca.1) {
            let ani_set = all_animations.0.get(akey).unwrap();
            transitions
            .play(
                &mut player,
                ani_set.animations[ca.0],
                Duration::from_millis(250),
            )
            .repeat();            
        }
    }
}

// ---
    
pub fn check(
    _tr: Trigger<OnAdd, CurrentAnimation>,
    animated_q: Query<&AnimationKey, Without<CurrentAnimation>>,
    check_q: Single<Entity, (With<NotReady>, With<TempAnimatorMarker>)>,
    mut cmd: Commands
) {
    if animated_q.is_empty() {
        cmd.entity(check_q.into_inner()).despawn();
    } 
}
