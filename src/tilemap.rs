#![allow(clippy::type_complexity)]
use std::collections::{HashMap, VecDeque};

use bevy::{prelude::*, transform::TransformBundle};

use crate::{
    chunk::{Chunk, ChunkBundle},
    tile::{Tile, TileBundle},
    util::{chunk_from_location, tile_from_location},
};

#[derive(Clone, Debug)]
pub enum TileEvent {
    MakeChunk(IVec2),
    SetTile {
        loc: IVec2,
        entity: Entity,
    },
    DeleteTile {
        loc: IVec2,
        mark: bool,
    },
    SetPixel {
        loc: IVec2,
        pixel: IVec2,
        color: Color,
    },
}

#[derive(Bundle, Default)]
pub struct TilemapBundle {
    tilemap: Tilemap,
    transform: TransformBundle,
    visibility: VisibilityBundle,
}

#[derive(Component, Default)]
pub struct Tilemap {
    chunks: HashMap<IVec2, Entity>,
    pub(crate) tasks: VecDeque<TileEvent>,
}

impl Tilemap {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            tasks: VecDeque::new(),
        }
    }

    pub fn require_chunk(&mut self, loc: IVec2) {
        let chunk = chunk_from_location(loc);

        if !self.chunks.contains_key(&IVec2::new(chunk.x, chunk.y)) {
            self.tasks
                .push_front(TileEvent::MakeChunk(IVec2::new(chunk.x, chunk.y)));
        }
    }

    pub fn set_tile(
        &mut self,
        commands: &mut Commands,
        loc: IVec2,
        tile: Tile,
        additional_components: impl Bundle,
    ) -> Entity {
        self.require_chunk(loc);

        let entity = commands
            .spawn((TileBundle::new(tile, loc), additional_components))
            .id();

        self.tasks.push_back(TileEvent::SetTile { loc, entity });

        entity
    }

    pub fn try_set_tile(
        &mut self,
        commands: &mut Commands,
        chunks: &Query<&Chunk>,
        loc: IVec2,
        tile: Tile,
        additional_components: impl Bundle,
    ) -> Option<Entity> {
        if self.get_tile(loc, chunks).is_some() {
            return None;
        }

        Some(self.set_tile(commands, loc, tile, additional_components))
    }

    pub fn set_pixel(&mut self, loc: IVec2, pixel: IVec2, color: Color) {
        if !self.has_chunk(loc) {
            return;
        }

        self.tasks
            .push_back(TileEvent::SetPixel { loc, pixel, color })
    }

    pub fn delete_tile(&mut self, loc: IVec2) {
        if !self.has_chunk(loc) {
            return;
        }

        self.tasks
            .push_back(TileEvent::DeleteTile { loc, mark: true });

        self.require_chunk(loc)
    }

    pub fn delete_without_marker(&mut self, loc: IVec2) {
        if !self.has_chunk(loc) {
            return;
        }

        self.tasks
            .push_back(TileEvent::DeleteTile { loc, mark: false });

        self.require_chunk(loc)
    }

    pub fn get_tile(&mut self, loc: IVec2, chunks: &Query<&Chunk>) -> Option<Entity> {
        if !self.has_chunk(loc) {
            return None;
        }

        if let Some(entity) = self.get_chunk(loc) {
            if let Ok(chunk) = chunks.get(entity) {
                return chunk.get_tile(tile_from_location(loc));
            }
        }

        None
    }

    pub fn get_chunk(&self, loc: IVec2) -> Option<Entity> {
        let loc = chunk_from_location(loc);
        self.chunks.get(&loc).copied()
    }

    pub fn has_chunk(&self, loc: IVec2) -> bool {
        let loc = chunk_from_location(loc);
        self.chunks.contains_key(&loc)
    }
}

pub fn tilemap_event_system(
    mut commands: Commands,
    mut tilemaps: Query<(Entity, &mut Tilemap)>,
    mut chunks: Query<(Entity, &mut Chunk)>,
    mut tiles: Query<(Entity, &mut Tile)>,
    mut images: ResMut<Assets<Image>>,
) {
    for (tilemap_entity, mut tilemap) in &mut tilemaps {
        let mut remaining_tasks = VecDeque::new();
        while let Some(event) = tilemap.tasks.pop_front() {
            match event {
                TileEvent::MakeChunk(loc) => {
                    if !tilemap.has_chunk(loc) {
                        let entity = commands
                            .spawn(ChunkBundle::new(
                                chunk_from_location(loc),
                                Chunk::new(&mut images),
                            ))
                            .id();

                        commands.entity(entity).set_parent(tilemap_entity);

                        tilemap.chunks.insert(chunk_from_location(loc), entity);
                    }
                }
                TileEvent::SetTile { loc, entity } => {
                    let chunk_loc = chunk_from_location(loc);

                    if !tilemap.has_chunk(loc) {
                        remaining_tasks.push_front(TileEvent::MakeChunk(loc));
                        remaining_tasks.push_back(TileEvent::SetTile { loc, entity });
                    } else if let Ok((chunk_entity, mut chunk)) =
                        chunks.get_mut(*tilemap.chunks.get(&chunk_loc).expect("chunk should exist"))
                    {
                        commands.entity(entity).set_parent(chunk_entity);
                        chunk.set_tile_entity(tile_from_location(loc), entity, &mut commands)
                    } else {
                        remaining_tasks.push_back(TileEvent::SetTile { loc, entity })
                    }
                }
                TileEvent::DeleteTile { loc, mark } => {
                    let chunk_loc = chunk_from_location(loc);

                    if tilemap.has_chunk(loc) {
                        if let Ok((_, mut chunk)) = chunks.get_mut(
                            *tilemap
                                .chunks
                                .get_mut(&chunk_loc)
                                .expect("Chunk should exist"),
                        ) {
                            if mark {
                                chunk.delete_tile(tile_from_location(loc), &mut commands);
                            } else {
                                chunk.delete_unmarked(tile_from_location(loc), &mut commands);
                            }
                        }
                    }
                }
                TileEvent::SetPixel { loc, pixel, color } => {
                    let chunk_loc = chunk_from_location(loc);

                    if tilemap.has_chunk(loc) {
                        if let Ok((_, mut chunk)) = chunks.get_mut(
                            *tilemap
                                .chunks
                                .get_mut(&chunk_loc)
                                .expect("Chunk should exist"),
                        ) {
                            if let Some(tile) = chunk.get_tile(tile_from_location(loc)) {
                                tiles
                                    .get_mut(tile)
                                    .expect("Tile should exist")
                                    .1
                                    .set_pixel(pixel, color);
                            }
                            chunk.update_tile(tile_from_location(loc));
                        }
                    }
                }
            }
        }
        tilemap.tasks.append(&mut remaining_tasks);
    }
}
