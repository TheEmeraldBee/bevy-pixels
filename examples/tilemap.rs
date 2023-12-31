use bevy::{prelude::*, render::camera::ScalingMode, utils::HashMap, window::PrimaryWindow};
use bevy_pixel_map::prelude::*;

use bevy_pixel_map::multi_tile::MultiTile;

#[derive(Default, Resource)]
pub struct TileResource {
    image_handles: HashMap<String, Handle<Image>>,
}

#[derive(Default, Resource)]
pub struct MousePosResource {
    pub mouse_pos: Vec2,
}

pub fn mouse_pos_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut mouse_pos_resource: ResMut<MousePosResource>,
) {
    if let Some(mouse_pos) = windows.single().cursor_position() {
        let (camera, trans) = cameras.single();
        let mouse_location = camera.viewport_to_world_2d(trans, mouse_pos).unwrap();
        mouse_pos_resource.mouse_pos = mouse_location;
    }
}

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PixelPlugin)
        .add_systems(Startup, setup_system)
        .add_systems(Update, (mouse_pos_system, placement_system).chain())
        .insert_resource(TileResource::default())
        .insert_resource(MousePosResource::default())
        .run()
}

pub fn setup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut tile_resource: ResMut<TileResource>,
) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.viewport_origin = Vec2::new(0.5, 0.5);
    camera_bundle.projection.scaling_mode = ScalingMode::WindowSize(25.6);
    commands.spawn(camera_bundle);

    commands.spawn(TilemapBundle::default());

    let image_resoure = asset_server.load("lightslate.png");
    tile_resource
        .image_handles
        .insert("Lightslate".to_string(), image_resoure);
}

fn placement_system(
    mut commands: Commands,
    mut tilemaps: Query<&mut Tilemap>,
    input: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    mouse_pos_resource: Res<MousePosResource>,

    images: Res<Assets<Image>>,
    tile_resource: Res<TileResource>,
) {
    // Now convert to tile coords.
    let (tile_coord, pixel_coord) = world_unit_to_pixel(mouse_pos_resource.mouse_pos);

    if input.pressed(MouseButton::Left) {
        let image = images
            .get(tile_resource.image_handles.get("Lightslate").unwrap())
            .unwrap();
        // Set the tile at that location
        tilemaps.single_mut().set_tile(
            &mut commands,
            tile_coord,
            Tile::from_image(image, (16..24, 16..24)),
            (),
        );
    }

    if input.pressed(MouseButton::Right) {
        tilemaps.single_mut().delete_tile(tile_coord);
    }

    if keys.pressed(KeyCode::P) {
        // Set the tile's pixel to red
        tilemaps
            .single_mut()
            .set_pixel(tile_coord, pixel_coord, Color::RED);
    }

    if keys.pressed(KeyCode::M) {
        // Remove the tile at that location
        let image = images
            .get(tile_resource.image_handles.get("Lightslate").unwrap())
            .unwrap();

        let tile = MultiTile::from_image(image);
        tile.place(tile_coord, &mut tilemaps.single_mut(), &mut commands);
    }
}
