use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(WordCursor(Vec2::ZERO))
        .add_systems(Startup, setup)
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
            ),
        )
        .run();
}

fn update_cursor(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut world_cursor: ResMut<WordCursor>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = camera_query.single();

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

const CARD_SIZE: Vec3 = Vec3::new(1., 1., 1.);

fn create_card(
    world_cursor: Res<WordCursor>,
    buttons: Res<Input<MouseButton>>,
    asset_server: Res<AssetServer>,
    commands: Commands,
) {

    if buttons.just_pressed(MouseButton::Right) {
        spawn_card(world_cursor.0, commands, asset_server);
    }

}

fn select_card(
    query: Query<(Entity, &Bounds, With<Card>)>,
    world_cursor: Res<WordCursor>,
    buttons: Res<Input<MouseButton>>,
    mut commands: Commands,
) {
    for (entity, bounds, _) in &query {
        if buttons.just_pressed(MouseButton::Left) && bounds.0.contains(world_cursor.0) {
            commands.entity(entity).insert(Selected);
        } else if buttons.just_released(MouseButton::Left) {
            commands.entity(entity).remove::<Selected>();
        }
    }
}

fn drag_selected(
    mut query: Query<(Entity, &mut Transform, With<Card>, With<Selected>)>,
    world_cursor: Res<WordCursor>,
    mut commands: Commands,
) {
    for (i, (entity, mut transform, _, _)) in query.iter_mut().enumerate() {
        let index = (i as f32) + 1.0;
        let offset = (i as f32) * 10.0;
        let dragging = Dragging(world_cursor.0 + offset);

        transform.translation = transform
            .translation
            .lerp(Vec3::new(offset + dragging.0.x, offset + dragging.0.y, index), 0.1 * index);

        transform.scale = transform.scale.lerp(CARD_SIZE * 1.2, 0.1);
        commands.entity(entity).insert(dragging);
    }
}

fn finish_drag_selected(
    mut query: Query<(
        Entity,
        &Dragging,
        &mut Transform,
        With<Card>,
        Without<Selected>,
    )>,
    mut commands: Commands,
) {
    for (entity, dragging, mut transform, _, _) in &mut query {
        if transform.translation.xy() == dragging.0 {
            commands.entity(entity).remove::<Dragging>();
        }

        transform.translation = transform
            .translation
            .lerp(Vec3::new(dragging.0.x, dragging.0.y, 0.0), 0.1);

        transform.scale = transform.scale.lerp(CARD_SIZE * 1.2, 0.1);
    }
}

fn non_selected(mut query: Query<(&mut Transform, With<Card>, Without<Selected>)>) {
    for (mut transform, _, _) in &mut query {
        transform.scale = transform.scale.lerp(CARD_SIZE, 0.2);
    }
}

#[derive(Resource)]
struct WordCursor(Vec2);

#[derive(Component)]
struct Card;

#[derive(Component)]
struct Dragging(Vec2);

#[derive(Component)]
struct Selected;

#[derive(Component)]
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

fn spawn_card(pos: Vec2, mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Card,
        Bounds(Rect::new(0.0, 0.0, 0.0, 0.0)),
        SpriteBundle {
            texture: asset_server.load("card_back.png"),
            transform: Transform::from_xyz(pos.x, pos.y, 0.).with_scale(CARD_SIZE),
            ..default()
        },
    ));
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    spawn_card(Vec2::new(0.0, 0.0), commands, asset_server);
}
