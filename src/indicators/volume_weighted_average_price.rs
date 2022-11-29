use std::fmt;

use crate::errors::{Result, TaError};
use crate::{DataItem, High, Low, Next, Period, Reset, Volume, Close};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum VolumeWeightedAveragePriceBands {
    Up,
    Down,
}

#[derive(Clone, Debug)]
pub enum VolumeWeightedAveragePriceSource {
    HLC3
}

// comments

#[doc(alias = "EMA")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct VolumeWeightedAveragePrice {
    cumulative_total: f64,
    cumulative_volume: f64,
    cumulative_v2: f64,
    vwap: f64,
    std_dev: f64,
    period: usize,
    source: VolumeWeightedAveragePriceSource
}

impl VolumeWeightedAveragePrice {
    pub fn new(period: usize) -> Result<Self> {
        match period {
            0 => Err(TaError::InvalidParameter),
            _ => Ok(Self {
                period,
                cumulative_total: 0.0,
                cumulative_volume: 0.0,
                cumulative_v2: 0.0,
                vwap: 0.0,
                std_dev: 0.0,
                source: VolumeWeightedAveragePriceSource::HLC3
            }),
        }
    }

    pub fn std_dev(&self, offset: f64, band_direction: VolumeWeightedAveragePriceBands) -> f64 {
        match band_direction {
            VolumeWeightedAveragePriceBands::Up => self.vwap + offset * self.std_dev,
            VolumeWeightedAveragePriceBands::Down => self.vwap - offset * self.std_dev,
        }
    }

    pub fn typical_price(&self, d: &DataItem) -> f64 {
        match self.source {
            VolumeWeightedAveragePriceSource::HLC3 => (d.high() + d.low() + d.close()) / 3.0,
        }
    }
}

impl Period for VolumeWeightedAveragePrice {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<DataItem> for VolumeWeightedAveragePrice {
    type Output = f64;

    fn next(&mut self, d: DataItem) -> Self::Output {
        let typical_price = self.typical_price(&d);

        self.cumulative_volume = d.volume() + self.cumulative_volume;

        self.cumulative_total = (typical_price * d.volume()) + self.cumulative_total;
        self.vwap = self.cumulative_total / self.cumulative_volume;

        self.cumulative_v2 = (d.volume() * typical_price * typical_price) + self.cumulative_v2;

        let val = (self.cumulative_v2 / self.cumulative_volume) - self.vwap * self.vwap;
        self.std_dev = val.max(0.0).sqrt();

        self.vwap
    }
}

impl Reset for VolumeWeightedAveragePrice {
    fn reset(&mut self) {
        self.cumulative_total = 0.0;
        self.cumulative_volume = 0.0;
        self.cumulative_v2 = 0.0;
        self.vwap = 0.0;
        self.std_dev = 0.0;
    }
}

impl Default for VolumeWeightedAveragePrice {
    fn default() -> Self {
        Self::new(14).unwrap()
    }
}

impl fmt::Display for VolumeWeightedAveragePrice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VWAP({})", self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use std::iter::zip;
    use VolumeWeightedAveragePriceBands::*;

    #[test]
    fn test_new() {
        assert!(VolumeWeightedAveragePrice::new(0).is_err());
        assert!(VolumeWeightedAveragePrice::new(1).is_ok());
    }

    fn generate_bar(record: (f64, f64, f64, f64)) -> DataItem {
        let (high, low, close, volume): (f64, f64, f64, f64) = record;
        // open doesn't matter in our context
        DataItem::builder()
            .open(high)
            .high(high)
            .low(low)
            .close(close)
            .volume(volume)
            .build()
            .unwrap()
    }

    #[test]
    fn test_next() {
        let bars = generate_bars_next();

        let mut vwap = VolumeWeightedAveragePrice::new(18).unwrap();

        let result = vec![
            150.31, 150.41, 150.41, 150.43, 150.56, 150.66, 150.66, 150.67, 150.70, 150.83, 150.85,
            150.91, 150.98, 151.05, 151.16, 151.18, 151.24,
        ];

        let iter = zip(result, bars);

        iter.for_each(|(r, b)| {
            assert_approx_eq!(vwap.next(b), r, 0.01);
        });
    }

    fn generate_bars_next() -> Vec<DataItem> {
        let values = vec![
            (150.39, 150.22, 150.31, 380.0),
            (150.47, 150.38, 150.41, 5270.0),
            (150.49, 150.33, 150.46, 990.0),
            (150.63, 150.44, 150.61, 1031.0),
            (151.10, 150.67, 151.01, 2675.0),
            (151.30, 150.77, 150.80, 3334.0),
            (150.95, 150.78, 150.93, 430.0),
            (151.12, 150.80, 151.10, 220.0),
            (151.27, 151.01, 151.25, 900.0),
            (151.35, 151.26, 151.33, 4088.0),
            (151.52, 151.32, 151.51, 650.0),
            (151.69, 151.49, 151.67, 1582.0),
            (152.03, 151.66, 151.80, 1892.0),
            (151.90, 151.75, 151.88, 2200.0),
            (152.15, 151.86, 152.10, 3043.0),
            (152.43, 152.03, 152.33, 675.0),
            (152.57, 152.25, 152.50, 1243.0),
        ];
        let bars: Vec<DataItem> = values
            .iter()
            .map(|bar_values| generate_bar(bar_values.to_owned()))
            .collect();
        bars
    }

    #[test]
    fn test_next_std_dev() {
        let bars = generate_bars_std_dev();

        let mut vwap = VolumeWeightedAveragePrice::new(18).unwrap();

        let result = vec![
            76.529,
            76.06681967213113,
            76.14020352035202,
            76.1069703043801,
            76.08272575250835,
            76.08949204052098,
            76.09489425587466,
            76.0890033076075,
            76.06792505995203,
            76.04638956433638,
            76.03966807214805,
            76.03571974110032,
            76.03484347469781,
            76.03699661971831,
            76.04293789029535,
            76.0455211312361,
        ];

        let result_vwap_offset_2 = vec![
            // upper, lower (bands)
            (76.529, 76.529),
            (76.15085245902783, 75.98278688523443),
            (76.27197010203601, 76.00843693866803),
            (76.25148729829992, 75.96245331046028), // DOWN
            (76.23361138086482, 75.93184012415189),
            (76.23399096258677, 75.94499311845519),
            (76.23600089361555, 75.95378761813377),
            (76.22155470994076, 75.95645190527424),
            (76.21690062084983, 75.91894949905424), // DOWN
            (76.21628813403485, 75.8764909946379),
            (76.20971995401032, 75.86961619028577),
            (76.20113288848688, 75.87030659371376),
            (76.19621376886145, 75.87347318053416),
            (76.19215131179875, 75.88184192763786),
            (76.19422672735257, 75.89164905323813),
            (76.19376091244948, 75.89728135002272),
        ];

        let result_vwap_offset_3 = vec![
            // upper, lower (bands)
            (76.529, 76.529),
            (76.19286885247618, 75.94077049178608),
            (76.33785339287802, 75.94255364782602),
            (76.32374579525984, 75.89019481350036),
            (76.30905419504305, 75.85639730997366),
            (76.30624042361966, 75.8727436574223),
            (76.30655421248599, 75.88323429926334),
            (76.28783041110741, 75.89017620410759),
            (76.29138840129872, 75.84446171860534),
            (76.30123741888409, 75.79154170978867),
            (76.29474589494147, 75.78459024935462),
            (76.28383946218017, 75.78760002002048),
            (76.27689891594328, 75.79278803345234),
            (76.26972865783898, 75.80426458159764),
            (76.2698711458812, 75.8160046347095),
            (76.26788080305617, 75.82316145941603),
        ];

        let iter = zip(zip(result, zip(result_vwap_offset_2, result_vwap_offset_3)), bars);

        iter.for_each(|((r, (rv2, rv3)), b)| {
            let (rv2u, rv2d) = rv2;
            let (rv3u, rv3d) = rv3;
            assert_approx_eq!(vwap.next(b), r, 0.01);
            assert_approx_eq!(vwap.std_dev(2.0, Up), rv2u, 0.01);
            assert_approx_eq!(vwap.std_dev(2.0, Down), rv2d, 0.01);
            assert_approx_eq!(vwap.std_dev(3.0, Up), rv3u, 0.01);
            assert_approx_eq!(vwap.std_dev(3.0, Down), rv3d, 0.01);
        });
    }

    fn generate_bars_std_dev() -> Vec<DataItem> {
        let values = vec![
            // high    ,low     ,close   ,Volume
            (76.529, 76.529, 76.529, 1.0),
            (76.073, 76.043, 76.073, 121.0),
            (76.323, 76.053, 76.193, 181.0),
            (76.208, 75.918, 75.988, 146.0),
            (76.088, 75.883, 76.058, 149.0),
            (76.183, 76.063, 76.153, 93.0),
            (76.178, 76.098, 76.158, 75.0),
            (76.153, 75.984, 76.034, 141.0),
            (76.092, 75.903, 75.929, 205.0),
            (76.017, 75.812, 75.958, 204.0),
            (76.012, 75.922, 75.937, 107.0),
            (76.049, 75.943, 75.977, 122.0),
            (76.069, 75.938, 76.048, 82.0),
            (76.117, 75.997, 76.068, 148.0),
            (76.167, 76.027, 76.093, 200.0),
            (76.109, 76.048, 76.104, 123.0),
        ];

        let bars: Vec<DataItem> = values
            .iter()
            .map(|bar_values| generate_bar(bar_values.to_owned()))
            .collect();
        bars
    }

    #[test]
    fn test_reset() {
        let mut vwap = VolumeWeightedAveragePrice::new(5).unwrap();

        assert_approx_eq!(
            vwap.next(generate_bar((150.39, 150.22, 150.31, 380.0))),
            150.31,
            0.01
        );
        vwap.next(generate_bar((150.47, 150.38, 150.41, 5270.0)));
        assert_ne!(
            vwap.next(generate_bar((150.49, 150.33, 150.46, 990.0))),
            150.31
        );

        vwap.reset();
        assert_approx_eq!(
            vwap.next(generate_bar((150.39, 150.22, 150.31, 380.0))),
            150.31,
            0.01
        );
    }

    #[test]
    fn test_default() {
        VolumeWeightedAveragePrice::default();
    }

    #[test]
    fn test_display() {
        let vwap = VolumeWeightedAveragePrice::new(7).unwrap();
        assert_eq!(format!("{}", vwap), "VWAP(7)");
    }
}
