use thiserror::Error;

/// A rectangular area of the sky to query.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    lat_min: f64,
    lon_min: f64,
    lat_max: f64,
    lon_max: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BoundingBoxError {
    #[error("latitudes must be within [-90, 90] and longitudes within [-180, 180]")]
    OutOfRange,
    #[error("each minimum must be strictly smaller than its maximum")]
    NotOrdered,
}

impl BoundingBox {
    pub fn new(
        lat_min: f64,
        lon_min: f64,
        lat_max: f64,
        lon_max: f64,
    ) -> Result<Self, BoundingBoxError> {
        let lat_ok = |v: f64| (-90.0..=90.0).contains(&v);
        let lon_ok = |v: f64| (-180.0..=180.0).contains(&v);

        if !(lat_ok(lat_min) && lat_ok(lat_max) && lon_ok(lon_min) && lon_ok(lon_max)) {
            return Err(BoundingBoxError::OutOfRange);
        }
        if lat_min >= lat_max || lon_min >= lon_max {
            return Err(BoundingBoxError::NotOrdered);
        }

        Ok(Self {
            lat_min,
            lon_min,
            lat_max,
            lon_max,
        })
    }

    pub fn area_square_degrees(&self) -> f64 {
        (self.lat_max - self.lat_min) * (self.lon_max - self.lon_min)
    }

    /// API credits one query over this area costs on OpenSky.
    ///
    /// The tiers come from the OpenSky REST API documentation. Check them
    /// there if they ever look wrong: they are the server's rules, not ours.
    pub fn credit_cost(&self) -> u32 {
        match self.area_square_degrees() {
            a if a <= 25.0 => 1,
            a if a <= 100.0 => 2,
            a if a <= 400.0 => 3,
            _ => 4,
        }
    }

    /// The query parameters OpenSky expects.
    pub(crate) fn query(&self) -> [(&'static str, f64); 4] {
        [
            ("lamin", self.lat_min),
            ("lomin", self.lon_min),
            ("lamax", self.lat_max),
            ("lomax", self.lon_max),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_valid_box() {
        assert!(BoundingBox::new(45.8389, 5.9962, 47.8229, 10.5226).is_ok());
    }

    #[test]
    fn rejects_out_of_range_and_nan() {
        assert_eq!(
            BoundingBox::new(-91.0, 0.0, 10.0, 10.0),
            Err(BoundingBoxError::OutOfRange)
        );
        assert_eq!(
            BoundingBox::new(0.0, 0.0, 10.0, 181.0),
            Err(BoundingBoxError::OutOfRange)
        );
        assert_eq!(
            BoundingBox::new(f64::NAN, 0.0, 10.0, 10.0),
            Err(BoundingBoxError::OutOfRange)
        );
    }

    #[test]
    fn rejects_inverted_or_empty_boxes() {
        assert_eq!(
            BoundingBox::new(10.0, 0.0, 5.0, 10.0),
            Err(BoundingBoxError::NotOrdered)
        );
        assert_eq!(
            BoundingBox::new(0.0, 5.0, 10.0, 5.0),
            Err(BoundingBoxError::NotOrdered)
        );
    }

    #[test]
    fn credit_cost_follows_the_area_tiers() {
        let cost = |h: f64, w: f64| BoundingBox::new(0.0, 0.0, h, w).unwrap().credit_cost();

        assert_eq!(cost(5.0, 5.0), 1); // 25 square degrees: the edge of tier 1
        assert_eq!(cost(5.0, 5.1), 2);
        assert_eq!(cost(10.0, 10.0), 2); // 100: the edge of tier 2
        assert_eq!(cost(20.0, 20.0), 3); // 400: the edge of tier 3
        assert_eq!(cost(30.0, 30.0), 4);
    }
}
