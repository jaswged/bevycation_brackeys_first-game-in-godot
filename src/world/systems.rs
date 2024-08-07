use bevy::prelude::*;
use bevy::text::{BreakLineOn, Text2dBounds};
use bevy_ecs_ldtk::{TileEnumTags};
use bevy_spritesheet_animation::component::SpritesheetAnimation;
use bevy_xpbd_2d::math::{Scalar, Vector};
use bevy_xpbd_2d::prelude::*;
use crate::{Player};
use crate::player::components::CoinCollected;
use crate::world::components::*;


pub(crate) fn add_colliders_to_walls_system(
    mut commands: Commands,
    wall_query: Query<Entity, (Added<Wall>, Without<Collider>)>,
) {
    for entity in wall_query.iter() {
        commands.entity(entity)
            .insert(Name::new("Wall"))
            .with_children(|commands| {
                commands.spawn((
                    TransformBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)),
                    Collider::rectangle(16.0, 16.0),
                    RigidBody::Static,
                    CollisionLayers::new(GamePhysicsLayer::Ground, [GamePhysicsLayer::Enemy, GamePhysicsLayer::Player])
                ));
            });
    }
}


pub fn add_colliders_to_platforms_system(
    mut commands: Commands,
    platform_query: Query<Entity, (Added<Platform>, Without<Collider>)>,
) {
    for entity in platform_query.iter() {
        commands.entity(entity)
            .insert(RigidBody::Kinematic)
            .insert(OneWayPlatform::default())
            .with_children(|commands| {
                commands.spawn((
                    Name::new("PlatformCollider"),
                    TransformBundle::from_transform(Transform::from_xyz(0.0, 4.0, 0.0)),
                    Collider::rectangle(32.0, 8.0),
                    CollisionLayers::new(GamePhysicsLayer::Ground, [GamePhysicsLayer::Enemy, GamePhysicsLayer::Player])
                ));
            });
    }
}


pub fn add_colliders_to_bridges_system(
    mut commands: Commands,
    platform_query: Query<(Entity, &TileEnumTags), (Added<Bridge>, Without<Collider>)>,
) {
    let collision_layer = CollisionLayers::new(
        GamePhysicsLayer::Ground,
        [GamePhysicsLayer::Enemy, GamePhysicsLayer::Player],
    );

    for (entity, enum_tag) in platform_query.iter() {
        commands.entity(entity)
            .insert((
                Name::new("Bridge"),
                RigidBody::Kinematic,
                Friction::new(1.0),
                OneWayPlatform::default(),
            ))
            .with_children(|commands| {
                if enum_tag.tags.contains(&"StartBridge".to_string()) {
                    commands.spawn((
                        TransformBundle::from_transform(Transform::from_xyz(0.0, 4.2, 0.0)
                            .with_rotation(Quat::from_rotation_z(-0.2))),
                        Collider::rectangle(16.0, 4.0),
                        collision_layer,
                    ));
                } else if enum_tag.tags.contains(&"MiddleBridge".to_string()) {
                    commands.spawn((
                        TransformBundle::from_transform(Transform::from_xyz(0.0, 2.5, 0.0)),
                        Collider::rectangle(16.0, 4.0),
                        collision_layer,
                    ));
                } else if enum_tag.tags.contains(&"EndBridge".to_string()) {
                    commands.spawn((
                        TransformBundle::from_transform(Transform::from_xyz(0.0, 4.2, 0.0)
                            .with_rotation(Quat::from_rotation_z(0.2))),
                        Collider::rectangle(16.0, 4.0),
                        collision_layer,
                    ));
                }
            });
    }
}

pub fn setup_coin_system(
    mut commands: Commands,
    coin_animations: Res<CoinAnimations>,
    coin_query: Query<(Entity, &Transform), (Added<Coin>, Without<SpritesheetAnimation>)>,
) {
    for (entity, transform) in coin_query.iter() {
        commands.entity(entity)
            .insert((
                SpriteSheetBundle {
                    texture: coin_animations.texture.clone_weak(),
                    atlas: TextureAtlas {
                        layout: coin_animations.layout.clone_weak(),
                        ..default()
                    },
                    transform: transform.clone(),
                    ..default()
                },
                SpritesheetAnimation::from_id(coin_animations.rotate_animation),
                RigidBody::Kinematic,
                Sensor,
                Collider::circle(5.0),
                CollisionLayers::new(GamePhysicsLayer::Collectible, [GamePhysicsLayer::Player])
            ));
    };
}

pub fn setup_score_display_system(
    mut commands: Commands,
    game_fonts: Res<GameFonts>,
    display_text_query: Query<(Entity, &Transform), (Added<ScoreDisplay>, Without<Text>)>,
) {
    for (entity, transform) in display_text_query.iter() {
        let box_size = Vec2::new(100.0, 50.0);

        commands.entity(entity)
            .insert(Text2dBundle {
                    transform: transform.clone(),
                    text: Text {
                        sections: vec![TextSection {
                            value: "You collected\n0 coins :(".to_owned(),
                            style: TextStyle {
                                font_size: 8.0,
                                font: game_fonts.pixelated_bold_font.clone_weak(),
                                color: Color::BLACK,
                            },
                        }],
                        justify: JustifyText::Center,
                        linebreak_behavior: BreakLineOn::WordBoundary,
                    },
                    text_2d_bounds: Text2dBounds {
                        // Wrap text in the rectangle
                        size: box_size,
                    },
                    ..default()
                });
    }
}

pub fn setup_tutorial_text_system(
    mut commands: Commands,
    game_fonts: Res<GameFonts>,
    tutorial_text_query: Query<(Entity, &TutorialText, &Transform), (Added<TutorialText>, Without<Text>)>,
) {
    for (entity, tutorial_text, transform) in tutorial_text_query.iter() {
        commands.entity(entity)
            .insert(Text2dBundle {
                    transform: transform.clone(),
                    text: Text::from_sections([
                        TextSection {
                            value: tutorial_text.text.to_owned(),
                            style: TextStyle {
                                font_size: 8.0,
                                font: game_fonts.pixelated_font.clone_weak(),
                                color: Color::BLACK,
                            },
                        }, ]),
                    ..default()
                });
    }
}


pub fn one_way_platform_system(
    mut one_way_platforms_query: Query<&mut OneWayPlatform>,
    other_colliders_query: Query<
        Option<&PassThroughOneWayPlatform>,
        (With<Collider>, Without<OneWayPlatform>), // NOTE: This precludes OneWayPlatform passing through a OneWayPlatform
    >,
    mut collisions: ResMut<Collisions>,
    collision_parent: Query<&ColliderParent>,
) {
    // This assumes that Collisions contains empty entries for entities
    // that were once colliding but no longer are.
    collisions.retain(|contacts| {
        // This is used in a couple of if statements below; writing here for brevity below.
        fn any_penetrating(contacts: &Contacts) -> bool {
            contacts.manifolds.iter().any(|manifold| {
                manifold
                    .contacts
                    .iter()
                    .any(|contact| contact.penetration > 0.0)
            })
        }

        // Differentiate between which normal of the manifold we should use
        enum RelevantNormal {
            Normal1,
            Normal2,
        }
        let entity1 = collision_parent.get(contacts.entity1).map(|p| p.get()).unwrap_or_else(|_| contacts.entity1);
        let entity2 = collision_parent.get(contacts.entity2).map(|p| p.get()).unwrap_or_else(|_| contacts.entity2);

        // First, figure out which entity is the one-way platform, and which is the other.
        // Choose the appropriate normal for pass-through depending on which is which.
        let (mut one_way_platform, other_entity, relevant_normal) =
            if let Ok(one_way_platform) = one_way_platforms_query.get_mut(entity1) {
                (one_way_platform, entity2, RelevantNormal::Normal1)
            } else if let Ok(one_way_platform) = one_way_platforms_query.get_mut(entity2) {
                (one_way_platform, entity1, RelevantNormal::Normal2)
            } else {
                // Neither is a one-way-platform, so accept the collision:
                // we're done here.
                return true;
            };

        if one_way_platform.0.contains(&other_entity) {
            // If we were already allowing a collision for a particular entity,
            // and if it is penetrating us still, continue to allow it to do so.
            if any_penetrating(contacts) {
                return false;
            } else {
                // If it's no longer penetrating us, forget it.
                one_way_platform.0.remove(&other_entity);
            }
        }

        match other_colliders_query.get(other_entity) {
            // Pass-through is set to never, so accept the collision.
            Ok(Some(PassThroughOneWayPlatform::Never)) => true,
            // Pass-through is set to always, so always ignore this collision
            // and register it as an entity that's currently penetrating.
            Ok(Some(PassThroughOneWayPlatform::Always)) => {
                one_way_platform.0.insert(other_entity);
                false
            }
            // Default behaviour is "by normal".
            Err(_) | Ok(None) | Ok(Some(PassThroughOneWayPlatform::ByNormal)) => {
                // If all contact normals are in line with the local up vector of this platform,
                // then this collision should occur: the entity is on top of the platform.
                if contacts.manifolds.iter().all(|manifold| {
                    let normal = match relevant_normal {
                        RelevantNormal::Normal1 => manifold.normal1,
                        RelevantNormal::Normal2 => manifold.normal2,
                    };

                    normal.length() > Scalar::EPSILON && normal.dot(Vector::Y) >= 0.5
                }) {
                    true
                } else if any_penetrating(contacts) {
                    // If it's already penetrating, ignore the collision and register
                    // the other entity as one that's currently penetrating.
                    one_way_platform.0.insert(other_entity);
                    false
                } else {
                    // In all other cases, allow this collision.
                    true
                }
            }
        }
    });
}

pub fn move_platforms_system(
    mut platform_query: Query<(&mut Transform, &mut LinearVelocity, &mut Path), With<Platform>>
) {
    for (mut transform, mut linvel, mut path) in platform_query.iter_mut() {
        if path.points.len() <= 1 { continue; };

        let next_point = path.points[path.index];
        let mut new_velocity =
            (next_point - transform.translation.truncate()).normalize() * path.speed;

        if new_velocity.dot(linvel.0) < 0. {
            if path.index == 0 {
                path.forward = true;
            } else if path.index == path.points.len() - 1 {
                path.forward = false;
            }

            transform.translation.x = path.points[path.index].x;
            transform.translation.y = path.points[path.index].y;

            if path.forward {
                path.index += 1;
            } else {
                path.index -= 1;
            }

            new_velocity =
                (path.points[path.index] - transform.translation.truncate()).normalize() * path.speed;
        }

        linvel.0 = new_velocity;
    }
}

pub fn kill_zone_system(
    mut commands: Commands,
    kill_zone_query: Query<&CollidingEntities, With<KillZone>>,
    mut player_query: Query<(Entity, &mut CollisionLayers), (With<Player>, Without<IsDead>)>,
    game_sounds: Res<GameSounds>
) {
    for collisions in kill_zone_query.iter() {
        for other in collisions.iter() {
            let Ok((_, mut collision_layers)) = player_query.get_mut(*other) else { continue };
            collision_layers.memberships = LayerMask::from(GamePhysicsLayer::Dead);

            commands.entity(*other).insert(IsDead);
            commands.spawn(AudioBundle {
                source: game_sounds.player_hurt.clone(),
                settings: PlaybackSettings::DESPAWN,
            });
        }
    }
}

pub fn update_score_display_system(
    mut coin_collected_events: EventReader<CoinCollected>,
    mut score_display_query: Query<&mut Text, With<ScoreDisplay>>,
) {
    let Ok(mut text) = score_display_query.get_single_mut() else { return };
    for coin_event in coin_collected_events.read() {
        let display_text = format!("You collected\n{} coins!", coin_event.total_collected);
        text.sections[0].value = display_text;
    }
    AudioBundle {
        settings: PlaybackSettings::ONCE,
        ..default()
    };
}

pub fn play_pickup_sound_system(
    mut commands: Commands,
    game_sounds: Res<GameSounds>,
    mut coin_collected_events: EventReader<CoinCollected>,
) {
    for _ in coin_collected_events.read() {
        commands.spawn(AudioBundle {
            source: game_sounds.coin_collected.clone(),
            settings: PlaybackSettings::DESPAWN,
        });
    }
}