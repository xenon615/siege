use bevy::prelude::*;

use crate::camera::Cam;
pub struct CameraTargetPlugin;
impl Plugin for CameraTargetPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, follow)
        .observe(set_target)
        ;
    }
}


#[derive(Component, Clone, Copy)]
pub struct CameraTarget {
    pub easing: bool,
    pub entity: Option<Entity>,
    pub position: Option<Vec3>,
    pub direction: Option<Dir3>,
    pub translation_bias: Vec3,
    pub rotation_bias: Vec3,
    pub velocity: f32
}

impl Default for CameraTarget {
    fn default() -> Self {
        CameraTarget {
            easing: true,
            entity: None,
            position: None,
            direction: None,
            translation_bias: Vec3::ZERO,
            rotation_bias: Vec3::ZERO,
            velocity: 5.
        }        
    }    
}

impl CameraTarget {

    pub fn from_entity(e: Entity) -> Self {
        CameraTarget {
            entity: Some(e),
            ..default()
        }
    }

    pub fn from_position(p: Vec3 ) -> Self {
        CameraTarget {
            position: Some(p),    
            ..default()
        }
    }
    #[allow(dead_code)]
    pub fn set_position(&mut self, p: Vec3){
        self.position = Some(p);
    }

    #[allow(dead_code)]
    pub fn with_direction(mut self, d: Dir3) -> Self {
        self.direction = Some(d);
        self
    }

    #[allow(dead_code)]
    pub fn set_direction(&mut self, d: Dir3){
        self.direction = Some(d);
    }

    pub fn with_translation_bias(mut self, b: Vec3) ->  Self {
        self.translation_bias = b;
        self
    }

    pub fn with_rotation_bias(mut self, b: Vec3) ->  Self {
        self.rotation_bias = b;
        self
    }

    pub fn with_velocity(mut self, v: f32) -> Self {
        self.velocity = v;
        self
    }
    #[allow(dead_code)]
    pub fn with_easing(mut self, e: bool) -> Self {
        self.easing = e;
        self
    }

}

#[derive(Event)]
pub struct SetCameraTarget(pub CameraTarget, pub isize);

// ---

fn set_target(
    trigger: Trigger<SetCameraTarget>,
    mut cmd: Commands,
    cams_q: Query<(Entity, &Camera)>
) {
    if let Some((cam_e, _)) =  cams_q.iter().find(|(_, c)|  c.order == trigger.event().1) {
        cmd.entity(cam_e).insert(trigger.event().0);
    }
}

// ---

fn follow (
    mut cam_q: Query<(&mut Transform, &CameraTarget), With<Cam>>,
    t_q: Query<&Transform, Without<Cam>>,
    time: Res<Time>,
) {
    for (mut cam_t, target) in cam_q.iter_mut() {

        let (pos, bias_t, bias_r) =  if let Some(ent) = target.entity {
            let Ok(tt) = t_q.get(ent) else {
                return;
            };
            (
                tt.translation, 
                tt.right() * target.translation_bias.x + tt.up() * target.translation_bias.y + tt.forward() * target.translation_bias.z,
                tt.right() * target.rotation_bias.x + tt.up() * target.rotation_bias.y + tt.forward() * target.rotation_bias.z
            )
        } else if let Some(pos) = target.position {
            (pos, target.translation_bias, target.rotation_bias)
        } else {
            return;
        };
        
        if target.easing {
            cam_t.translation = cam_t.translation.lerp(pos + bias_t, time.delta_seconds() * target.velocity);
        } else {
            let m = ((pos + bias_t) - cam_t.translation).normalize() * target.velocity * time.delta_seconds() * 100.;
            cam_t.translation += m;
        }
        
        let qq = if let Some(direction) = target.direction {
            cam_t.looking_to(direction, Vec3::Y)
        } else {
            cam_t.looking_at(pos + bias_r, Vec3::Y)        
        };

        cam_t.rotation = cam_t.rotation.slerp(qq.rotation, time.delta_seconds() * 5.);
    }

}
