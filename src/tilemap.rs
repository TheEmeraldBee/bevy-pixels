use std::{cmp::Ordering, collections::VecDeque};

use bevy::{
    prelude::{
        Assets, BuildChildren, Bundle, Commands, Component, ComputedVisibility, Entity, IVec2,
        Image, Query, ResMut, Vec2, Visibility,
    },
    transform::TransformBundle,
    utils::HashMap,
};

use crate::{
    chunk::{Chunk, ChunkBundle},
    tile::{Tile, TileBundle},
    CHUNK_SIZE,
};

#[derive(Clone, Debug)]
pub enum TileEvent {
    MakeChunk(IVec2),
    SetTile { loc: IVec2, entity: Entity },
    DeleteTile { loc: IVec2, mark: bool },
}

#[derive(Bundle)]
pub struct TilemapBundle {
    tilemap: Tilemap,
    transform: TransformBundle,
    computed_visibility: ComputedVisibility,
    visibility: Visibility,
}

impl Default for TilemapBundle {
    fn default() -> Self {
        Self {
            tilemap: Tilemap::default(),
            transform: TransformBundle::default(),
            computed_visibility: ComputedVisibility::default(),
            visibility: Visibility::Visible,
        }
    }
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

pub fn align_loc_to_chunk(mut loc: i32) -> i32 {
    match loc.cmp(&0) {
        Ordering::Greater => loc / CHUNK_SIZE as i32,
        Ordering::Less => {
            let mut result = 0;
            while loc < 0 {
                loc += CHUNK_SIZE as i32;
                result -= 1;
            }
            result
        }
        Ordering::Equal => 0,
    }
}

pub fn chunk_from_location(loc: IVec2) -> IVec2 {
    IVec2::new(align_loc_to_chunk(loc.x), align_loc_to_chunk(loc.y))
}

pub fn tile_from_location(loc: IVec2) -> IVec2 {
    let loc_x = loc.x.rem_euclid(CHUNK_SIZE as i32) as usize;
    let loc_y = loc.y.rem_euclid(CHUNK_SIZE as i32) as usize;

    IVec2::new(loc_x as i32, loc_y as i32)
}

// Converts a world coordinate to a tile location
pub fn world_unit_to_tile(loc: Vec2) -> IVec2 {
    IVec2::new(
        (loc.x).ceil() as i32 + CHUNK_SIZE as i32 / 2 - 1,
        (loc.y).ceil() as i32 + CHUNK_SIZE as i32 / 2 - 1,
    )
}

pub fn tilemap_event_system(
    mut commands: Commands,
    mut tilemaps: Query<(Entity, &mut Tilemap)>,
    mut chunks: Query<(Entity, &mut Chunk)>,
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
            }
        }
        tilemap.tasks.append(&mut remaining_tasks);
    }
}
