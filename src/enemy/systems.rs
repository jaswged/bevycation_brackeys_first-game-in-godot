use bevy::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use bevy_xpbd_2d::prelude::*;
use crate::enemy::components::{Enemy, MovementDirection, SlimeAnimations};
use crate::world::components::{GamePhysicsLayer, KillZone};

pub fn setup_enemy_system(
    mut commands: Commands,
    slime_animations: Res<SlimeAnimations>,
    enemy_query: Query<(Entity, &Transform), (Added<Enemy>, Without<SpritesheetAnimation>)>,
) {
    for (entity, transform) in enemy_query.iter() {
        commands.entity(entity)
            .insert((
                SpriteSheetBundle {
                    texture: slime_animations.texture.clone(),
                    atlas: TextureAtlas {
                        layout: slime_animations.layout.clone(),
                        ..default()
                    },
                    transform: transform.clone(),
                    ..default()
                },
                SpritesheetAnimation::from_id(slime_animations.idle),
                RigidBody::Dynamic,
                LockedAxes::ROTATION_LOCKED,
                GravityScale(10.0),
                LinearVelocity(Vec2::new(30.0, 0.0)),
                // Friction::new(0.0),
                RayCaster::new(Vec2::ZERO, Direction2d::X)
                    .with_max_time_of_impact(7.50)
                    // .with_ignore_self(true)
                    .with_solidness(true)
                    .with_max_hits(2)
                ,
                MovementDirection(Direction2d::X),
                KillZone,
                CollisionLayers::new(GamePhysicsLayer::Enemy, [GamePhysicsLayer::Enemy, GamePhysicsLayer::Player, GamePhysicsLayer::Ground])

            )).with_children(|commands| {
            commands.spawn((
                TransformBundle::from_transform(Transform::from_xyz(0.0, -2.5, 0.0)),
                // Collider::rectangle(8.0, 8.0),
                Collider::circle(5.0),
                Friction::new(0.0).with_combine_rule(CoefficientCombine::Min),
                KillZone,
            ));
        });
    }
}

pub fn enemy_wall_detection_system(
    mut enemy_query: Query<(Entity, &mut RayCaster, &RayHits, &mut Sprite, &mut MovementDirection), With<Enemy>>,
    collider_parent: Query<&ColliderParent>,
) {
    for (entity, mut raycaster, hits, mut sprite, mut movement_direction) in enemy_query.iter_mut() {
        for hit in hits.iter() {
            //ignore self collisions with parent rigidbody
            if collider_parent.get(hit.entity).map_or(false, |parent| parent.get() == entity) { continue; }
            raycaster.direction = -raycaster.direction;
            movement_direction.0 = -movement_direction.0;
            sprite.flip_x = sprite.flip_x;
            break;
        }
    }
}

pub fn enemy_movement_system(
    mut enemy_query: Query<(&mut LinearVelocity, &MovementDirection), With<Enemy>>,
) {
    for (mut linear_velocity, movement_direction) in enemy_query.iter_mut() {
        if linear_velocity.x.abs() < 20.0 {
            linear_velocity.0 += *movement_direction.0 * 1.0;
        }
    }
}
