use std::ops::Range;

use bevy::prelude::{Bundle, Color, Component, IVec2, Image, Transform};

use crate::TILE_SIZE;

#[derive(Component)]
pub struct DeletingTile;

#[derive(Bundle)]
pub struct TileBundle {
    transform: Transform,
    tile: Tile,
}

impl TileBundle {
    pub fn new(tile: Tile, location: IVec2) -> Self {
        Self {
            tile,
            transform: Transform::from_xyz(location.x as f32, location.y as f32, 0.0),
        }
    }
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Tile {
    pub(crate) pixels: [[Color; TILE_SIZE]; TILE_SIZE],
}

impl Tile {
    pub fn from_color(color: Color) -> Self {
        Self {
            pixels: [[color; TILE_SIZE]; TILE_SIZE],
        }
    }

    pub fn from_pixels(pixels: [[Color; TILE_SIZE]; TILE_SIZE]) -> Self {
        Self { pixels }
    }

    pub fn from_image(image: &Image, pixel_range: (Range<usize>, Range<usize>)) -> Self {
        let mut colors = [[Color::NONE; TILE_SIZE]; TILE_SIZE];

        assert_eq!(pixel_range.0.len(), TILE_SIZE);
        assert_eq!(pixel_range.1.len(), TILE_SIZE);

        for x in pixel_range.0.clone() {
            for y in pixel_range.1.clone() {
                let pixel_index = y * image.size().x as usize * 4 + x * 4;

                colors[y - pixel_range.1.start][x - pixel_range.0.start] = Color::rgba_u8(
                    image.data[pixel_index],
                    image.data[pixel_index + 1],
                    image.data[pixel_index + 2],
                    image.data[pixel_index + 3],
                );
            }
        }

        Self { pixels: colors }
    }

    pub fn set_pixel(&mut self, loc: IVec2, color: Color) {
        if !verify_pixel_loc(loc) {
            return;
        }

        self.pixels[loc.y as usize][loc.x as usize] = color
    }

    pub fn get_pixel(&self, loc: IVec2) -> Option<Color> {
        if !verify_pixel_loc(loc) {
            return None;
        }

        Some(self.pixels[loc.y as usize][loc.x as usize])
    }

    pub fn pixel_count(&self) -> usize {
        let mut pixel_count = 0;
        for x in 0..TILE_SIZE {
            for y in 0..TILE_SIZE {
                if self.pixels[y][x].a() > 0.0 {
                    pixel_count += 1;
                }
            }
        }

        pixel_count
    }
}

fn verify_pixel_loc(loc: IVec2) -> bool {
    if loc.x < 0 || loc.x >= TILE_SIZE as i32 || loc.y < 0 || loc.y >= TILE_SIZE as i32 {
        return false;
    }
    true
}
