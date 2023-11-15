use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    math::{vec2, vec3},
    prelude::*,
    render::view::RenderLayers,
    utils::HashMap,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .insert_resource(WordCursor(Vec2::ZERO))
        .add_systems(Startup, setup)
        //.add_plugins(bevy_editor_pls::EditorPlugin::default())
        .add_systems(
            Update,
            (
                update_cursor,
                update_bounds,
                drag_selected,
                finish_drag_selected,
                non_selected,
                select_card,
                create_card,
                show_piles,
                align_placed,
                animate_sprite,
                move_player_system,
            ),
        )
        .run();
}

fn update_cursor(
    camera_query: Query<(&Camera, &GlobalTransform, With<CardsCamera>)>,
    windows: Query<&Window>,
    mut world_cursor: ResMut<WordCursor>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform, _) = camera_query.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    // Calculate a world position based on the cursor's position.
    let Some(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    world_cursor.0 = point;

    gizmos.circle_2d(point, 10., Color::WHITE);
}

const CARD_SIZE: Vec3 = Vec3::new(0.5, 0.5, 1.0);

fn align_grid(bounds: &Bounds, offset: Vec2) -> Vec2 {
    ((bounds.center() * 1.0 / bounds.0.size()).floor() * bounds.0.size())
        + bounds.half_size()
        + offset
}

#[derive(Default)]
struct SpawnCounter(usize);

fn create_card(
    world_cursor: Res<WordCursor>,
    buttons: Res<Input<MouseButton>>,
    asset_server: Res<AssetServer>,
    mut counter: Local<SpawnCounter>,
    commands: Commands,
) {
    if buttons.just_pressed(MouseButton::Right) {
        let colors = [
            "card_back_blue.png",
            "card_back_purple.png",
            "card_back_red.png",
        ];

        counter.0 += 1;
        counter.0 %= colors.len();

        spawn_card(world_cursor.0, colors[counter.0], commands, asset_server);
    }
}

type SelectedCard = (With<Card>, With<Selected>);
type UnselectedCard = (With<Card>, Without<Selected>);

fn select_card(
    query: Query<(Entity, &Bounds, With<Card>)>,
    world_cursor: Res<WordCursor>,
    buttons: Res<Input<MouseButton>>,
    mut commands: Commands,
) {
    for (entity, bounds, _) in &query {
        if buttons.just_pressed(MouseButton::Left) && bounds.0.contains(world_cursor.0) {
            commands.entity(entity).insert(Selected);
            commands.entity(entity).remove::<Pile>();
        } else if buttons.just_released(MouseButton::Left) {
            commands.entity(entity).remove::<Selected>();
        }
    }
}

fn drag_selected(
    mut query: Query<(Entity, &mut Transform, &Bounds, SelectedCard)>,
    world_cursor: Res<WordCursor>,
    mut commands: Commands,
    mut gizmos: Gizmos,
) {
    for (i, (entity, mut transform, bounds, _)) in query.iter_mut().enumerate() {
        let index = (i as f32) + 1.0;
        let offset = (i as f32) * 10.0;

        let dragging = Dragging(world_cursor.0);

        if i == 0 {
            let target_bounds = Bounds(Rect::from_center_size(world_cursor.0, bounds.size()));
            let grid_pos = align_grid(&target_bounds, Vec2::ZERO);
            gizmos.rect_2d(grid_pos, 0.0, target_bounds.size(), Color::WHITE);
        }

        transform.translation = transform.translation.lerp(
            Vec3::new(offset + world_cursor.0.x, offset + world_cursor.0.y, index),
            0.1 * index,
        );

        transform.scale = transform.scale.lerp(CARD_SIZE * 1.2, 0.1);
        commands.entity(entity).insert(dragging);
    }
}

fn show_piles(query: Query<(&Pile, &Bounds)>, mut gizmos: Gizmos) {
    let mut pile_counts = HashMap::new();
    for (pile, _bounds) in &query {
        pile_counts.entry(pile).and_modify(|c| *c += 1).or_insert(1);
    }

    for (pile, bounds) in &query {
        let count = pile_counts[pile];
        if count > 1 {
            gizmos.circle_2d(pile.pos() + bounds.half_size(), 20.0, Color::ORANGE_RED);
        }
    }
}

fn finish_drag_selected(
    mut query: Query<(Entity, &Dragging, &mut Transform, UnselectedCard)>,
    mut commands: Commands,
) {
    for (entity, dragging, mut transform, _) in &mut query {
        if transform.translation.xy().floor() == dragging.0.floor() {
            println!("finished dragging: {:?}", entity);
            commands.entity(entity).remove::<Dragging>();
            commands.entity(entity).insert(Pile::new(dragging.0));
        }

        transform.translation = transform
            .translation
            .lerp(Vec3::new(dragging.0.x, dragging.0.y, 0.0), 0.15);

        transform.scale = transform.scale.lerp(CARD_SIZE, 0.15);
    }
}

fn non_selected(mut query: Query<(&mut Transform, With<Card>, Without<Selected>)>) {
    for (mut transform, _, _) in &mut query {
        transform.scale = transform.scale.lerp(CARD_SIZE, 0.2);
    }
}

fn align_placed(mut query: Query<(&Bounds, &mut Dragging, UnselectedCard)>) {
    for (bounds, mut dragging, _) in &mut query {
        dragging.0 = align_grid(
            &Bounds(Rect::from_center_size(dragging.0, bounds.size() * 1.2)),
            Vec2::ZERO,
        );
    }
}

#[derive(Resource, Deref)]
struct WordCursor(Vec2);

#[derive(Component)]
struct Card;

#[derive(Component, PartialEq, Eq, Hash)]
struct Pile(i32, i32);

impl Pile {
    fn new(pos: Vec2) -> Self {
        Self(pos.x as i32, pos.y as i32)
    }

    fn pos(&self) -> Vec2 {
        vec2(self.0 as f32, self.1 as f32)
    }
}

#[derive(Component, Deref)]
struct Dragging(Vec2);

#[derive(Component)]
struct Selected;

#[derive(Component, Deref)]
struct Bounds(Rect);

fn update_bounds(
    mut query: Query<(&Transform, &Handle<Image>, &mut Bounds, With<Sprite>)>,
    assets: Res<Assets<Image>>,
) {
    for (transform, image_handle, mut bounds, _) in query.iter_mut() {
        let Some(image_dimensions) = assets.get(image_handle) else {
            continue;
        };

        let scaled_image_dimension = image_dimensions.size_f32() * transform.scale.truncate();
        let bounding_box =
            Rect::from_center_size(transform.translation.truncate(), scaled_image_dimension);

        bounds.0 = bounding_box;
    }
}

fn spawn_card(pos: Vec2, card: &str, mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Card,
        Dragging(pos),
        RenderLayers::layer(0),
        Bounds(Rect::new(0.0, 0.0, 100.0, 100.0)),
        SpriteBundle {
            texture: asset_server.load(card.to_string()),
            transform: Transform::from_xyz(0., 0., 0.).with_scale(CARD_SIZE),
            ..default()
        },
    ));
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index >= indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("adventurer-sheet.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(50.0, 37.0), 7, 10, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    // Use only the subset of sprites in the sheet that make up the run animation
    let run_animation_indices = AnimationIndices { first: 8, last: 13 };

    spawn_player(&mut commands, texture_atlas_handle, run_animation_indices);
    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                // disable clearing completely (pixels stay as they are)
                // (preserves output from previous frame or camera/pass)
                clear_color: ClearColorConfig::None,
            },
            camera: Camera {
                order: 1,
                ..default()
            },
            ..default()
        },
        RenderLayers::from_layers(&[0]),
        CardsCamera,
    ));

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 0,
                ..default()
            },
            ..default()
        },
        RenderLayers::from_layers(&[1]),
        PlayerCamera,
    ));
}

fn move_player_system(
    mut query: Query<(&mut Transform, &mut AnimationIndices, With<Player>)>,
    keys: Res<Input<KeyCode>>,
) {
    let (mut player_transform, mut anim, _) = query.single_mut();

    let mut velocity = Vec2::ZERO;

    if keys.pressed(KeyCode::W) {
        velocity.y = 1.0;
    }

    if keys.pressed(KeyCode::S) {
        velocity.y = -1.0;
    }

    if keys.pressed(KeyCode::A) {
        velocity.x = -1.0;
    }

    if keys.pressed(KeyCode::D) {
        velocity.x = 1.0;
    }

    *anim = AnimationIndices { first: 8, last: 13 };
    if velocity.x < 0.0 {
        player_transform.scale.x = -3.0;
    } else if velocity.x > 0.0 {
        player_transform.scale.x = 3.0;
    } else {
        *anim = AnimationIndices { first: 0, last: 3 };
    }

    player_transform.translation += (velocity.normalize_or_zero() * 10.0).extend(0.0);
}

#[derive(Component)]
struct CardsCamera;

#[derive(Component)]
struct PlayerCamera;

#[derive(Component)]
struct Player;

fn spawn_player(
    commands: &mut Commands,
    texture_atlas_handle: Handle<TextureAtlas>,
    animation_indices: AnimationIndices,
) {
    commands.spawn((
        Player,
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(animation_indices.first),
            transform: Transform::from_scale(Vec3::splat(3.0))
                .with_translation(vec3(0.0, 0.0, 0.0)),
            ..default()
        },
        RenderLayers::layer(1),
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}
