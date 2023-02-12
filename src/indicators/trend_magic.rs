use std::fmt;

use crate::{Close, Next, Reset, High, Low};

use super::{TrueRange, SimpleMovingAverage, CommodityChannelIndex};

pub struct TrendMagic {
  atr_multiplier: f64,
  previous: f64,
  tr: TrueRange,
  tr_sma: SimpleMovingAverage,
  cci: CommodityChannelIndex
}

impl TrendMagic {
  pub fn new(atr_periods: usize, atr_multiplier: f64, cci_periods: usize) -> Self {
      Self {
        atr_multiplier,
        previous: 0.0,
        tr: TrueRange::new(),
        tr_sma: SimpleMovingAverage::new(atr_periods).unwrap(),
        cci: CommodityChannelIndex::new(cci_periods).unwrap()
      }
  }
}

impl<T: High + Low + Close> Next<&T> for TrendMagic {
  type Output = (f64, f64, f64);

  fn next(&mut self, input: &T) -> Self::Output {
    // tr and then sma the tr to act like an alternative to atr
    let tr = self.tr.next(input);
    let tr_sma = self.tr_sma.next(tr);
    let atr = tr_sma;
    // cci
    let cci = self.cci.next(input);
    // up or down
    let up = input.low() - atr * self.atr_multiplier;
    let down = input.high() + atr * self.atr_multiplier;
    if cci >= 0.0 {
      if up < self.previous {
        
      } else {
        self.previous = up;
      }
    } else {
      if down > self.previous {
        
      } else {
        self.previous = down;
      }
    }
    return (up, down, self.previous);
  }
}

impl Default for TrendMagic {
  fn default() -> Self {
      Self::new(5, 1.0, 20)
  }
}

impl fmt::Display for TrendMagic {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(f, "Trend Magic by KivancOzbilgic")
  }
}

impl Reset for TrendMagic {
  fn reset(&mut self) {
    self.previous = 0.0;
    self.tr.reset();
    self.tr_sma.reset();
    self.cci.reset();
  }
}
