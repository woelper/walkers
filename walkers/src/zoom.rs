#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[error("invalid zoom level")]
pub struct InvalidZoom;

#[derive(Debug, Clone, Copy)]
pub struct Zoom(f32);

impl TryFrom<f32> for Zoom {
    type Error = InvalidZoom;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        // Mapnik supports zooms up to 19.
        // https://wiki.openstreetmap.org/wiki/Slippy_map_tilenames#Zoom_levels
        if !(0. ..=19.).contains(&value) {
            Err(InvalidZoom)
        } else {
            Ok(Self(value))
        }
    }
}

impl Default for Zoom {
    fn default() -> Self {
        Self(16.)
    }
}

impl Zoom {
    pub fn round(&self) -> u8 {
        self.0.round() as u8
    }

    pub fn zoom_in(&mut self) -> Result<(), InvalidZoom> {
        *self = Self::try_from(self.0 + 1.)?;
        Ok(())
    }

    pub fn zoom_out(&mut self) -> Result<(), InvalidZoom> {
        *self = Self::try_from(self.0 - 1.)?;
        Ok(())
    }

    /// Zoom using a relative value.
    pub fn zoom_by(&mut self, value: f32) {
        if let Ok(new_self) = Self::try_from(self.0 + value) {
            *self = new_self;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructing_zoom() {
        assert_eq!(16, Zoom::default().round());
        assert_eq!(19, Zoom::try_from(19.).unwrap().round());
        assert_eq!(InvalidZoom, Zoom::try_from(20.).unwrap_err());
    }

    #[test]
    fn test_zooming_in() {
        let mut zoom = Zoom::try_from(18.).unwrap();
        assert!(zoom.zoom_in().is_ok());
        assert_eq!(19, zoom.round());
        assert_eq!(Err(InvalidZoom), zoom.zoom_in());
    }

    #[test]
    fn test_zooming_out() {
        let mut zoom = Zoom::try_from(1.).unwrap();
        assert!(zoom.zoom_out().is_ok());
        assert_eq!(0, zoom.round());
        assert_eq!(Err(InvalidZoom), zoom.zoom_out());
    }
}
