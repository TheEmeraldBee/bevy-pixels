use std::cmp::Ordering;

use bevy::prelude::{IVec2, Vec2};

use crate::{CHUNK_SIZE, TILE_SIZE};

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

/// Converts a world coordinate to a tile location
pub fn world_unit_to_tile(loc: Vec2) -> IVec2 {
    IVec2::new(
        (loc.x).ceil() as i32 + CHUNK_SIZE as i32 / 2 - 1,
        (loc.y).ceil() as i32 + CHUNK_SIZE as i32 / 2 - 1,
    )
}

/// Converts a world coordinate to a tile & pixel location
pub fn world_unit_to_pixel(loc: Vec2) -> (IVec2, IVec2) {
    let world_loc = world_unit_to_tile(loc);

    let mut leftover_loc = Vec2::new(
        world_loc.x as f32 - loc.x - 7.0,
        world_loc.y as f32 - loc.y - 7.0,
    );
    leftover_loc.x *= TILE_SIZE as f32;
    leftover_loc.y *= TILE_SIZE as f32;

    let pixel = IVec2::new(
        8 - (leftover_loc.x.ceil() as i32),
        leftover_loc.y.ceil() as i32 - 1,
    );

    (world_loc, pixel)
}
