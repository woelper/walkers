//! Project the lat/lon coordinates into a 2D x/y using the Web Mercator.
//! <https://en.wikipedia.org/wiki/Web_Mercator_projection>
//! <https://wiki.openstreetmap.org/wiki/Slippy_map_tilenames>
//! <https://www.netzwolf.info/osm/tilebrowser.html?lat=51.157800&lon=6.865500&zoom=14>

// zoom level   tile coverage  number of tiles  tile size(*) in degrees
// 0            1 tile         1 tile           360° x 170.1022°
// 1            2 × 2 tiles    4 tiles          180° x 85.0511°
// 2            4 × 4 tiles    16 tiles         90° x [variable]

/// Geographical position with latitude and longitude.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position(geo_types::Point);

impl Position {
    /// Construct from latitude and longitude.
    pub fn from_lat_lon(lat: f64, lon: f64) -> Self {
        Self(geo_types::Point::new(lon, lat))
    }

    /// Construct from longitude and latitude. Note that it is common standard to write coordinates
    /// starting with the latitude instead (e.g. `51.104465719934176, 17.075169894118684` is
    /// the [Wrocław's zoo](https://zoo.wroclaw.pl/en/)).
    pub fn from_lon_lat(lon: f64, lat: f64) -> Self {
        Self(geo_types::Point::new(lon, lat))
    }

    pub fn lat(&self) -> f64 {
        self.0.y()
    }

    pub fn lon(&self) -> f64 {
        self.0.x()
    }

    /// Project geographical position into a 2D plane using Mercator.
    pub fn project(&self, zoom: u8) -> Pixels {
        let (x, y) = mercator_normalized(*self);

        // Map that into a big bitmap made out of web tiles.
        let number_of_pixels = 2u32.pow(zoom as u32) * TILE_SIZE;
        let x = x * number_of_pixels as f64;
        let y = y * number_of_pixels as f64;

        Pixels::new(x, y)
    }

    /// Tile this position is on.
    pub(crate) fn tile_id(&self, mut zoom: u8, tile_size: u32) -> TileId {
        let (x, y) = mercator_normalized(*self);

        // Some providers provide larger tiles, effectively bundling e.g. 4 256px tiles in one
        // 512px one. To use this functionality, we zoom out correspondigly so the resolution
        // remains the same.
        let tile_size_correction = ((tile_size as f64) / (TILE_SIZE as f64)).log2();
        zoom -= tile_size_correction as u8;

        // Map that into a big bitmap made out of web tiles.
        let number_of_tiles = 2u32.pow(zoom as u32);
        let x = (x * number_of_tiles as f64).floor() as u32;
        let y = (y * number_of_tiles as f64).floor() as u32;

        TileId { x, y, zoom }
    }
}

impl From<geo_types::Point> for Position {
    fn from(value: geo_types::Point) -> Self {
        Self(value)
    }
}

impl From<Position> for geo_types::Point {
    fn from(value: Position) -> Self {
        value.0
    }
}

/// Location projected on the screen or an abstract bitmap.
pub type Pixels = geo_types::Point;

use std::f64::consts::PI;

pub trait PixelsExt {
    fn to_vec2(&self) -> egui::Vec2;
}

impl PixelsExt for Pixels {
    fn to_vec2(&self) -> egui::Vec2 {
        egui::Vec2::new(self.x() as f32, self.y() as f32)
    }
}

/// Size of the tiles used by the services like the OSM.
pub(crate) const TILE_SIZE: u32 = 256;

fn mercator_normalized(position: Position) -> (f64, f64) {
    // Project into Mercator (cylindrical map projection).
    let x = position.lon().to_radians();
    let y = position.lat().to_radians().tan().asinh();

    // Scale both x and y to 0-1 range.
    let x = (1. + (x / PI)) / 2.;
    let y = (1. - (y / PI)) / 2.;

    (x, y)
}

/// Coordinates of the OSM-like tile.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct TileId {
    /// X number of the tile.
    pub x: u32,

    /// Y number of the tile.
    pub y: u32,

    /// Zoom level, where 0 means no zoom.
    /// See: https://wiki.openstreetmap.org/wiki/Zoom_levels
    pub zoom: u8,
}

impl TileId {
    /// Tile position (in pixels) on the "World bitmap".
    pub fn project(&self, tile_size: u32) -> Pixels {
        Pixels::new((self.x * tile_size) as f64, (self.y * tile_size) as f64)
    }

    pub fn east(&self) -> Option<TileId> {
        Some(TileId {
            x: self.x + 1,
            y: self.y,
            zoom: self.zoom,
        })
    }

    pub fn west(&self) -> Option<TileId> {
        Some(TileId {
            x: self.x.checked_sub(1)?,
            y: self.y,
            zoom: self.zoom,
        })
    }

    pub fn north(&self) -> Option<TileId> {
        Some(TileId {
            x: self.x,
            y: self.y.checked_sub(1)?,
            zoom: self.zoom,
        })
    }

    pub fn south(&self) -> Option<TileId> {
        Some(TileId {
            x: self.x,
            y: self.y + 1,
            zoom: self.zoom,
        })
    }
}

/// Transforms screen pixels into a geographical position.
pub fn screen_to_position(pixels: Pixels, zoom: u8) -> Position {
    let number_of_pixels = 2u32.pow(zoom as u32) * TILE_SIZE;
    let number_of_pixels: f64 = number_of_pixels.into();

    let lon = pixels.x();
    let lon = lon / number_of_pixels;
    let lon = (lon * 2. - 1.) * PI;
    let lon = lon.to_degrees();

    let lat = pixels.y();
    let lat = lat / number_of_pixels;
    let lat = (-lat * 2. + 1.) * PI;
    let lat = lat.sinh().atan().to_degrees();

    Position::from_lon_lat(lon, lat)
}


/// Transforms screen pixels into a geographical position.
pub fn position_to_screen(pixels: Pixels, zoom: u8) -> Position {
    let number_of_pixels = 2u32.pow(zoom as u32) * TILE_SIZE;
    let number_of_pixels: f64 = number_of_pixels.into();

    let lon = pixels.x();
    let lon = lon / number_of_pixels;
    let lon = (lon * 2. - 1.) * PI;
    let lon = lon.to_degrees();

    let lat = pixels.y();
    let lat = lat / number_of_pixels;
    let lat = (-lat * 2. + 1.) * PI;
    let lat = lat.sinh().atan().to_degrees();

    Position::from_lon_lat(lon, lat)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projecting_position_and_tile() {
        let citadel = Position::from_lon_lat(21.00027, 52.26470);

        // Just a bit higher than what most providers support,
        // to make sure we cover the worst case in terms of precision.
        let zoom = 20;

        assert_eq!(
            TileId {
                x: 585455,
                y: 345104,
                zoom
            },
            citadel.tile_id(zoom, 256)
        );

        // Automatically zooms out for larger tiles
        assert_eq!(
            TileId {
                x: 292727,
                y: 172552,
                zoom: zoom - 1
            },
            citadel.tile_id(zoom, 512)
        );

        // Projected tile is just its x, y multiplied by the size of tiles.
        assert_eq!(
            Pixels::new(585455. * 256., 345104. * 256.),
            citadel.tile_id(zoom, 256).project(256)
        );

        // Projected Citadel position should be somewhere near projected tile, shifted only by the
        // position on the tile.
        let calculated = citadel.project(zoom);
        let citadel_proj = Pixels::new(585455. * 256. + 184., 345104. * 256. + 116.5);
        approx::assert_relative_eq!(calculated.x(), citadel_proj.x(), max_relative = 0.5);
        approx::assert_relative_eq!(calculated.y(), citadel_proj.y(), max_relative = 0.5);
    }

    #[test]
    fn project_there_and_back() {
        let citadel = Position::from_lat_lon(21.00027, 52.26470);
        let zoom = 16;
        let calculated = screen_to_position(citadel.project(zoom), zoom);

        approx::assert_relative_eq!(calculated.lon(), citadel.lon(), max_relative = 1.0);
        approx::assert_relative_eq!(calculated.lat(), citadel.lat(), max_relative = 1.0);
    }

    #[test]
    /// Just to be compatible with the `geo` ecosystem.
    fn position_is_compatible_with_geo_types() {
        let original = Position::from_lat_lon(21.00027, 52.26470);
        let converted: geo_types::Point = original.into();
        let brought_back: Position = converted.into();

        approx::assert_relative_eq!(original.lon(), brought_back.lon());
        approx::assert_relative_eq!(original.lat(), brought_back.lat());
    }
}
