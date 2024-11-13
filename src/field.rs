use bevy::prelude::*;
pub struct FieldPlugin;
impl Plugin for FieldPlugin {
    fn build(&self, app: &mut App) {
        app
        .register_type::<FieldTarget>()
        .add_systems(Startup, startup)
        ;
    }
}

// ---

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct FieldTarget;


// ---

fn startup(
    mut cmd: Commands,
    assets: ResMut<AssetServer>
) {
    cmd.spawn((
        SceneBundle {
            scene: assets.load(GltfAssetLabel::Scene(0).from_asset("models/field.glb")),
            ..default()
        },
     ));
}

// ---

