use bevy::{prelude::*, render::render_resource::Extent3d};

use crate::{
    tile::{DeletingTile, Tile},
    CHUNK_SIZE, TILE_SIZE,
};

#[derive(Bundle)]
pub struct ChunkBundle {
    sprite: SpriteBundle,
    chunk: Chunk,
}

impl ChunkBundle {
    pub fn new(loc: IVec2, chunk: Chunk) -> Self {
        Self {
            sprite: SpriteBundle {
                texture: chunk.image_handle.clone_weak(),
                transform: Transform::from_xyz(
                    loc.x as f32 * CHUNK_SIZE as f32,
                    loc.y as f32 * CHUNK_SIZE as f32,
                    0.0,
                )
                .with_scale(Vec3::splat(1.0 / TILE_SIZE as f32)),
                ..Default::default()
            },
            chunk,
        }
    }
}

#[derive(Component)]
pub struct Chunk {
    tiles: [[Option<Entity>; CHUNK_SIZE]; CHUNK_SIZE],
    image: Image,
    image_handle: Handle<Image>,
    dirty_tiles: Vec<IVec2>,
}

impl Chunk {
    pub fn new(images: &mut ResMut<Assets<Image>>) -> Self {
        let mut data = vec![];

        for _ in 0..((CHUNK_SIZE * CHUNK_SIZE) * TILE_SIZE * TILE_SIZE) {
            data.append(&mut Color::rgba(0.0, 0.0, 0.0, 0.0).as_rgba_u8().to_vec());
        }

        let image = Image::new(
            Extent3d {
                width: CHUNK_SIZE as u32 * TILE_SIZE as u32,
                height: CHUNK_SIZE as u32 * TILE_SIZE as u32,
                ..default()
            },
            bevy::render::render_resource::TextureDimension::D2,
            data,
            bevy::render::render_resource::TextureFormat::Rgba8Unorm,
        );

        Self {
            tiles: [[None; CHUNK_SIZE]; CHUNK_SIZE],
            image: image.clone(),
            image_handle: images.add(image),
            dirty_tiles: vec![],
        }
    }

    pub fn get_tile(&self, loc: IVec2) -> Option<Entity> {
        if !verify_chunk_loc(loc) {
            return None;
        }

        self.tiles[loc.x as usize][loc.y as usize]
    }

    pub fn set_tile(
        &mut self,
        my_entity: Entity,
        loc: IVec2,
        tile: Tile,
        additional_components: impl Bundle,
        commands: &mut Commands,
    ) {
        if !verify_chunk_loc(loc) {
            return;
        }

        self.delete_tile(loc, commands);

        self.tiles[loc.x as usize][loc.y as usize] = Some(
            commands
                .spawn((tile, additional_components))
                .set_parent(my_entity)
                .id(),
        );

        self.update_tile(loc)
    }

    pub fn set_tile_entity(&mut self, loc: IVec2, entity: Entity, commands: &mut Commands) {
        self.delete_tile(loc, commands);
        self.tiles[loc.x as usize][loc.y as usize] = Some(entity);

        self.update_tile(loc)
    }

    pub fn delete_tile(&mut self, loc: IVec2, commands: &mut Commands) {
        if let Some(entity) = self.get_tile(loc) {
            commands.entity(entity).insert(DeletingTile);
            self.tiles[loc.x as usize][loc.y as usize] = None;
            self.update_tile(loc);
        }
    }

    pub fn delete_unmarked(&mut self, loc: IVec2, commands: &mut Commands) {
        if let Some(entity) = self.get_tile(loc) {
            commands.entity(entity).despawn_recursive();
            self.tiles[loc.x as usize][loc.y as usize] = None;
            self.update_tile(loc);
        }
    }

    pub fn update_tile(&mut self, loc: IVec2) {
        if !verify_chunk_loc(loc) {
            return;
        }
        if self.dirty_tiles.contains(&loc) {
            return;
        }

        self.dirty_tiles.push(loc)
    }

    pub fn update_texture(
        &mut self,
        images: &mut ResMut<Assets<Image>>,
        tiles: &mut Query<&mut Tile>,
    ) {
        if self.dirty_tiles.is_empty() {
            return;
        }

        let mut data = self.image.data.clone();

        for loc in &self.dirty_tiles {
            if let Some(tile) = self.get_tile(*loc) {
                if let Ok(tile) = tiles.get(tile) {
                    for pixel_x in 0..TILE_SIZE {
                        for pixel_y in 0..TILE_SIZE {
                            // Inverse of y * pixels per tile + the current pixel
                            let pixel_index_y =
                                ((CHUNK_SIZE - 1 - loc.y as usize) * TILE_SIZE) + pixel_y;
                            // x * pixels per tile + the current pixel
                            let pixel_index_x = (loc.x as usize * TILE_SIZE) + pixel_x;

                            let color = tile
                                .get_pixel(IVec2::new(pixel_x as i32, pixel_y as i32))
                                .expect("Pixel should be in range")
                                .as_rgba_u8();

                            // Update the color on the texture with tile texture
                            data[pixel_index_y * (CHUNK_SIZE * TILE_SIZE) * 4
                                + pixel_index_x * 4] = color[0];

                            data[(pixel_index_y * (CHUNK_SIZE * TILE_SIZE) * 4
                                + pixel_index_x * 4)
                                + 1] = color[1];

                            data[(pixel_index_y * (CHUNK_SIZE * TILE_SIZE) * 4
                                + pixel_index_x * 4)
                                + 2] = color[2];

                            data[(pixel_index_y * (CHUNK_SIZE * TILE_SIZE) * 4
                                + pixel_index_x * 4)
                                + 3] = color[3];
                        }
                    }
                }
            } else {
                // It's dirty, but the tile doesn't exist, so clear the tile's color.
                for pixel_x in 0..TILE_SIZE {
                    for pixel_y in 0..TILE_SIZE {
                        // Inverse of y * pixels per tile + the current pixel
                        let pixel_index_y =
                            ((CHUNK_SIZE - 1 - loc.y as usize) * TILE_SIZE) + pixel_y;

                        // x * pixels per tile + the current pixel
                        let pixel_index_x = (loc.x as usize * TILE_SIZE) + pixel_x;

                        // Update the color on the texture with tile texture
                        data[pixel_index_y * (CHUNK_SIZE * TILE_SIZE) * 4 + pixel_index_x * 4] = 0;

                        data[(pixel_index_y * (CHUNK_SIZE * TILE_SIZE) * 4 + pixel_index_x * 4)
                            + 1] = 0;

                        data[(pixel_index_y * (CHUNK_SIZE * TILE_SIZE) * 4 + pixel_index_x * 4)
                            + 2] = 0;

                        data[(pixel_index_y * (CHUNK_SIZE * TILE_SIZE) * 4 + pixel_index_x * 4)
                            + 3] = 0;
                    }
                }
            }
        }

        self.dirty_tiles.clear();

        self.image.data = data;

        images.insert(self.image_handle.clone(), self.image.clone());
    }
}

fn verify_chunk_loc(loc: IVec2) -> bool {
    if loc.x < 0 || loc.x >= CHUNK_SIZE as i32 || loc.y < 0 || loc.y >= CHUNK_SIZE as i32 {
        return false;
    }
    true
}

pub fn chunk_texture_update(
    mut images: ResMut<Assets<Image>>,
    mut tiles: Query<&mut Tile>,
    mut chunks: Query<&mut Chunk>,
) {
    for mut chunk in &mut chunks {
        chunk.update_texture(&mut images, &mut tiles)
    }
}

pub fn chunk_deleter(
    mut commands: Commands,
    deleting_tiles: Query<Entity, (With<Tile>, With<DeletingTile>)>,
) {
    for entity in &deleting_tiles {
        commands.entity(entity).despawn_recursive();
    }
}
