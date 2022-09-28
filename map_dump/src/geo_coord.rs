use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

// A (latitude, longitude) pair.
// Latitude is in the range [-90, 90] and positive values correspond to the
// hemisphere.
// Longitude is in the range [-180, 180], where higher values go towards the
// west and lower values go towards the east.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeoCoord(pub f64, pub f64);

impl GeoCoord {
    pub fn neighborhood(&self, step_size: f64) -> (Self, Self) {
        (
            GeoCoord(self.0 - step_size, self.1 - step_size).clamp(),
            GeoCoord(self.0 + step_size, self.1 + step_size).clamp(),
        )
    }
    pub fn clamp(&self) -> Self {
        Self(self.0.clamp(-90.0, 90.0), self.1.clamp(-180.0, 180.0))
    }

    pub fn mid(&self, other: &Self) -> Self {
        Self((self.0 + other.0) / 2.0, (self.1 + other.1) / 2.0)
    }
}

#[derive(Clone, Debug)]
pub struct VecGeoCoord(f64, f64, f64);

impl VecGeoCoord {
    pub fn cos_geo_dist(&self, other: &Self) -> f64 {
        self.0 * other.0 + self.1 * other.1 + self.2 * other.2
    }
}

impl From<GeoCoord> for VecGeoCoord {
    fn from(x: GeoCoord) -> VecGeoCoord {
        let (lat, lon) = (x.0 * PI / 180.0, x.1 * PI / 180.0);
        let z = lat.sin();
        let radius = lat.cos();
        let x = radius * lon.cos();
        let y = radius * lon.sin();
        VecGeoCoord(x, y, z)
    }
}

pub struct GeoBounds(pub GeoCoord, pub GeoCoord);

impl GeoBounds {
    pub fn globe(step_size: f64) -> Vec<GeoBounds> {
        let mut all_regions = Vec::new();
        let mut lat = 0.0;
        while lat < 90.0 {
            let mut lon = -180.0;
            while lon < 180.0 {
                for lat_sign in [-1.0, 1.0] {
                    let coord = GeoCoord(lat * lat_sign, lon);
                    let (min, max) = coord.neighborhood(step_size);
                    all_regions.push(GeoBounds(min, max));
                    lon += step_size;
                }
            }
            lat += step_size;
        }
        all_regions
    }

    pub fn mid(&self) -> GeoCoord {
        return self.0.mid(&self.1);
    }

    pub fn split(&self) -> [GeoBounds; 4] {
        let x0 = self.0 .0;
        let y0 = self.0 .1;
        let x1 = self.mid().0;
        let y1 = self.mid().1;
        let x2 = self.1 .0;
        let y2 = self.1 .1;
        [
            GeoBounds(GeoCoord(x0, y0), GeoCoord(x1, y1)),
            GeoBounds(GeoCoord(x1, y0), GeoCoord(x2, y1)),
            GeoBounds(GeoCoord(x0, y1), GeoCoord(x1, y2)),
            GeoBounds(GeoCoord(x1, y1), GeoCoord(x2, y2)),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::{GeoCoord, VecGeoCoord};
    use std::f64::consts::PI;

    #[test]
    fn geo_dist() {
        let cases = [
            (GeoCoord(0.0, 10.0), GeoCoord(0.0, 30.0), 20.0 * PI / 180.0),
            (GeoCoord(10.0, -20.0), GeoCoord(-15.0, 100.0), 2.118313649),
        ];
        for (p1, p2, expected) in cases {
            let actual = VecGeoCoord::from(p1)
                .cos_geo_dist(&VecGeoCoord::from(p2))
                .acos();
            println!("{}, {}", actual, expected);
            assert!((actual - expected).abs() < 1e-5);
        }
    }

    #[test]
    fn vec_geo_coord_from() {
        for gc in [
            GeoCoord(0.0, 0.0),
            GeoCoord(10.0, -20.0),
            GeoCoord(80.0, 70.0),
        ] {
            let v = VecGeoCoord::from(gc);
            assert!((v.cos_geo_dist(&v) - 1.0).abs() < 1e-8);
        }
    }
}
