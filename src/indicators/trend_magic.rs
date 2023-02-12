use std::fmt;

use crate::{Close, Next, Reset, High, Low};

use super::{TrueRange, SimpleMovingAverage, CommodityChannelIndex};

pub struct TrendMagic {
  atr_multiplier: f64,
  previous_trend_magic: f64,
  previous_low: f64,
  previous_high: f64,
  previous_close: f64,
  tr: TrueRange,
  tr_sma: SimpleMovingAverage,
  cci: CommodityChannelIndex
}

impl TrendMagic {
  pub fn new(atr_periods: usize, atr_multiplier: f64, cci_periods: usize) -> Self {
      Self {
        atr_multiplier,
        previous_trend_magic: 0.0,
        previous_close: 0.0,
        previous_high: 0.0,
        previous_low: 0.0,
        tr: TrueRange::new(),
        tr_sma: SimpleMovingAverage::new(atr_periods).unwrap(),
        cci: CommodityChannelIndex::new(cci_periods).unwrap()
      }
  }
}

fn crossover(previous_x: f64, previous_y: f64, current_x: f64, current_y: f64) -> bool {
  let condition1 = previous_x <= previous_y;
  let condition2 = current_x >= current_y;
  return condition1 && condition2;
}

fn crossunder(previous_x: f64, previous_y: f64, current_x: f64, current_y: f64) -> bool {
  let condition1 = previous_x >= previous_y;
  let condition2 = current_x <= current_y;
  return condition1 && condition2;
}

fn cross(previous_x: f64, previous_y: f64, current_x: f64, current_y: f64) -> bool {
  let crossover_result = crossover(previous_x, previous_y, current_x, current_y);
  let crossunder_result = crossover(previous_x, previous_y, current_x, current_y);
  return crossover_result || crossunder_result;
}

impl<T: High + Low + Close> Next<&T> for TrendMagic {
  type Output = (bool, bool, bool);

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
    let current_trend_magic = if cci >= 0.0 {
      if up < self.previous_trend_magic {
        self.previous_trend_magic
      } else {
        up
      }
    } else {
      if down > self.previous_trend_magic {
        self.previous_trend_magic
      } else {
        down
      }
    };
    // crosses
    let cross1 = cross(input.close(), current_trend_magic, self.previous_close, self.previous_trend_magic);
    let cross2 = crossover(input.low(), current_trend_magic, self.previous_low, self.previous_trend_magic);
    let cross3 = crossunder(input.high(), current_trend_magic, self.previous_high, self.previous_trend_magic);
    // update state
    self.previous_trend_magic = current_trend_magic;
    self.previous_close = input.close();
    self.previous_high = input.high();
    self.previous_low = input.low();
    return (
      cross1,
      cross2,
      cross3
    )
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
    self.previous_close = 0.0;
    self.previous_high = 0.0;
    self.previous_low = 0.0;
    self.previous_trend_magic = 0.0;
    self.tr.reset();
    self.tr_sma.reset();
    self.cci.reset();
  }
}
