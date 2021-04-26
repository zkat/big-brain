/*!
Utilities for turning values within a certain range into different curves.
*/

/**
Trait that any evaluators must implement. Must return an `f32` value between `0.0..=100.0`.
 */
pub trait Evaluator: std::fmt::Debug + Sync + Send + Clone {
    fn evaluate(&self, value: f32) -> f32;
}

/**
[`Evaluator`] for linear values. That is, there's no curve to the value mapping.
 */
#[derive(Debug, Clone)]
pub struct LinearEvaluator {
    xa: f32,
    ya: f32,
    yb: f32,
    dy_over_dx: f32,
}

impl LinearEvaluator {
    pub fn new() -> Self {
        Self::new_full(0.0, 0.0, 1.0, 1.0)
    }
    pub fn new_inversed() -> Self {
        Self::new_ranged(1.0, 0.0)
    }
    pub fn new_ranged(min: f32, max: f32) -> Self {
        Self::new_full(min, 0.0, max, 1.0)
    }
    fn new_full(xa: f32, ya: f32, xb: f32, yb: f32) -> Self {
        Self {
            xa,
            ya,
            yb,
            dy_over_dx: (yb - ya) / (xb - xa),
        }
    }
}

impl Default for LinearEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for LinearEvaluator {
    fn evaluate(&self, value: f32) -> f32 {
        clamp(
            self.ya + self.dy_over_dx * (value - self.xa),
            self.ya,
            self.yb,
        )
    }
}

/**
[`Evaluator`] with an exponent curve. The value will grow according to its `power` parameter.
 */
#[derive(Debug, Clone)]
pub struct PowerEvaluator {
    xa: f32,
    ya: f32,
    xb: f32,
    power: f32,
    dy: f32,
}

impl PowerEvaluator {
    pub fn new(power: f32) -> Self {
        Self::new_full(power, 0.0, 0.0, 1.0, 1.0)
    }
    pub fn new_ranged(power: f32, min: f32, max: f32) -> Self {
        Self::new_full(power, min, 0.0, max, 1.0)
    }
    fn new_full(power: f32, xa: f32, ya: f32, xb: f32, yb: f32) -> Self {
        Self {
            power: clamp(power, 0.0, 10000.0),
            dy: yb - ya,
            xa,
            ya,
            xb,
        }
    }
}

impl Default for PowerEvaluator {
    fn default() -> Self {
        Self::new(2.0)
    }
}

impl Evaluator for PowerEvaluator {
    fn evaluate(&self, value: f32) -> f32 {
        let cx = clamp(value, self.xa, self.xb);
        self.dy * ((cx - self.xa) / (self.xb - self.xa)).powf(self.power) + self.ya
    }
}

/**
[`Evaluator`] with a "Sigmoid", or "S-like" curve.
 */
#[derive(Debug, Clone)]
pub struct SigmoidEvaluator {
    xa: f32,
    xb: f32,
    ya: f32,
    yb: f32,
    k: f32,
    two_over_dx: f32,
    x_mean: f32,
    y_mean: f32,
    dy_over_two: f32,
    one_minus_k: f32,
}

impl SigmoidEvaluator {
    pub fn new(k: f32) -> Self {
        Self::new_full(k, 0.0, 0.0, 1.0, 1.0)
    }

    pub fn new_ranged(k: f32, min: f32, max: f32) -> Self {
        Self::new_full(k, min, 0.0, max, 1.0)
    }

    fn new_full(k: f32, xa: f32, ya: f32, xb: f32, yb: f32) -> Self {
        let k = clamp(k, -0.99999, 0.99999);
        Self {
            xa,
            xb,
            ya,
            yb,
            two_over_dx: (2.0 / (xb - ya)).abs(),
            x_mean: (xa + xb) / 2.0,
            y_mean: (ya + yb) / 2.0,
            dy_over_two: (yb - ya) / 2.0,
            one_minus_k: 1.0 - k,
            k,
        }
    }
}

impl Evaluator for SigmoidEvaluator {
    fn evaluate(&self, x: f32) -> f32 {
        let cx_minus_x_mean = clamp(x, self.xa, self.xb) - self.x_mean;
        let numerator = self.two_over_dx * cx_minus_x_mean * self.one_minus_k;
        let denominator = self.k * (1.0 - 2.0 * (self.two_over_dx * cx_minus_x_mean)).abs() + 1.0;
        clamp(
            self.dy_over_two * (numerator / denominator) + self.y_mean,
            self.ya,
            self.yb,
        )
    }
}

impl Default for SigmoidEvaluator {
    fn default() -> Self {
        Self::new(-0.5)
    }
}

pub(crate) fn clamp<T: PartialOrd>(val: T, min: T, max: T) -> T {
    let val = if val > max { max } else { val };
    if val < min {
        min
    } else {
        val
    }
}
