use bevy::prelude::*;

use crate::{
    chunk::Chunk,
    tile::{DeletingTile, Tile},
    tilemap::Tilemap,
    TILE_SIZE,
};

#[derive(Component, Clone, Debug)]
pub struct MultiTileMarker {
    entity: Entity,
}

#[derive(Component, Clone, Debug)]
pub struct MultiTile {
    pos: IVec2,
    size: IVec2,
    pixels: Vec<Vec<Color>>,
    entities: Vec<Entity>,
}

impl MultiTile {
    pub fn from_color(color: Color, width: usize, height: usize) -> Self {
        let data = vec![vec![color; width]; height];

        Self {
            pixels: data,
            pos: IVec2::new(0, 0),
            size: IVec2::new(0, 0),
            entities: vec![],
        }
    }

    pub fn from_image(image: &Image) -> Self {
        let mut pixels = vec![];

        for y in 0..image.size().y as usize {
            let mut row = vec![];
            for x in 0..image.size().x as usize {
                let pixel_index = y * image.size().x as usize * 4 + x * 4;

                row.push(Color::rgba_u8(
                    image.data[pixel_index],
                    image.data[pixel_index + 1],
                    image.data[pixel_index + 2],
                    image.data[pixel_index + 3],
                ));
            }
            pixels.push(row)
        }

        let size = IVec2::new(
            pixels.len() as i32 / TILE_SIZE as i32,
            pixels[0].len() as i32 / TILE_SIZE as i32,
        );

        Self {
            pixels,
            pos: IVec2::new(0, 0),
            size,
            entities: vec![],
        }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn get_tile(&self, offset: IVec2) -> Option<Tile> {
        if offset.x < 0 || offset.x > self.size.x || offset.y < 0 || offset.y > self.size.y {
            return None;
        }

        let mut pixels = [[Color::NONE; TILE_SIZE]; TILE_SIZE];

        for y in 0..TILE_SIZE {
            for x in 0..TILE_SIZE {
                pixels[y][x] = self.pixels[y + offset.y as usize * TILE_SIZE]
                    [x + offset.x as usize * TILE_SIZE];
            }
        }

        Some(Tile::from_pixels(pixels))
    }

    pub fn place(mut self, loc: IVec2, tilemap: &mut Tilemap, commands: &mut Commands) -> Entity {
        self.pos.x = loc.x;
        self.pos.y = loc.y;

        let mut my_entity = commands.spawn_empty();
        let entity_id = my_entity.id();
        // Create the tiles
        for tile_x in 0..self.size.x {
            for tile_y in 0..self.size.y {
                let entity = tilemap.set_tile(
                    commands,
                    IVec2::new(loc.x + tile_x, loc.y + tile_y),
                    self.get_tile(IVec2::new(tile_x, self.size.y - 1 - tile_y))
                        .unwrap(),
                    MultiTileMarker { entity: entity_id },
                );
                self.entities.push(entity)
            }
        }

        my_entity = commands.entity(entity_id);

        my_entity.insert(self);
        my_entity.id()
    }
}

pub fn multi_tile_delete(
    mut commands: Commands,
    mut deleting_tiles: Query<(&Parent, &MultiTileMarker), With<DeletingTile>>,
    multi_tiles: Query<(Entity, &MultiTile)>,
    chunk_data: Query<&Chunk>,
    chunks: Query<&Parent, With<Chunk>>,
    mut tilemaps: Query<&mut Tilemap>,
) {
    let mut handled_multi_tiles = vec![];
    for (chunk, marker) in &mut deleting_tiles {
        if let Ok((multi_entity, multi_tile)) = multi_tiles.get(marker.entity) {
            if handled_multi_tiles.contains(&multi_tile.pos) {
                continue;
            }
            handled_multi_tiles.push(multi_tile.pos);

            commands.entity(multi_entity).despawn_recursive();

            if let Ok(tilemap) = chunks.get(chunk.get()) {
                if let Ok(mut tilemap) = tilemaps.get_mut(tilemap.get()) {
                    for x in 0..multi_tile.size.x {
                        for y in 0..multi_tile.size.y {
                            let loc = IVec2::new(x + multi_tile.pos.x, y + multi_tile.pos.y);
                            if let Some(tile_entity) = tilemap.get_tile(loc, &chunk_data) {
                                if multi_tile.entities.contains(&tile_entity) {
                                    tilemap.delete_tile(loc);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
