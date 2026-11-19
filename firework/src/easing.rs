// Часть проекта Firework с открытым исходным кодом.
// Лицензия EPL 2.0, подробнее в файле LICENSE. Copyright (c) 2026 Firework

use core::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
pub struct EasingRaw {
    pub func: fn(f32) -> f32,
}

impl EasingRaw {
    #[inline(always)]
    pub const fn new(func: fn(f32) -> f32) -> Self {
        Self { func }
    }

    /// Вычисляет значение кривой с ограничением прогресса t в диапазоне [0.0, 1.0]
    #[inline(always)]
    pub fn eval(&self, t: f32) -> f32 {
        (self.func)(t.clamp(0.0, 1.0))
    }

    /// Вычисляет значение кривой без ограничения t
    #[inline(always)]
    pub fn eval_unclamped(&self, t: f32) -> f32 {
        (self.func)(t)
    }

    /// Линейная интерполяция от start к end с применением кривой плавности
    #[inline(always)]
    pub fn lerp(&self, start: f32, end: f32, t: f32) -> f32 {
        let eased = self.eval(t);
        start + (end - start) * eased
    }
}

/// Набор функций плавности для анимаций
#[derive(Debug, Clone, Copy, Default)]
pub enum Easing {
    #[default]
    Linear,
    InSine,
    OutSine,
    InOutSine,
    InQuad,
    OutQuad,
    InOutQuad,
    InCubic,
    OutCubic,
    InOutCubic,
    InQuart,
    OutQuart,
    InOutQuart,
    InQuint,
    OutQuint,
    InOutQuint,
    InExpo,
    OutExpo,
    InOutExpo,
    InCirc,
    OutCirc,
    InOutCirc,
    InBack,
    OutBack,
    InOutBack,
    InElastic,
    OutElastic,
    InOutElastic,
    InBounce,
    OutBounce,
    InOutBounce,
    SmoothStep,
    SmootherStep,
    StepStart,
    StepEnd,

    /// Пользовательская функция плавности
    Custom(fn(f32) -> f32),
}

impl Easing {
    /// Возвращает EasingRaw с указателем на соответствующую функцию кривой. Можно вызывать 
    /// в const контексте
    #[inline]
    pub const fn to_raw(self) -> EasingRaw {
        let func: fn(f32) -> f32 = match self {
            Easing::Linear => linear,

            Easing::InSine => in_sine,
            Easing::OutSine => out_sine,
            Easing::InOutSine => in_out_sine,

            Easing::InQuad => in_quad,
            Easing::OutQuad => out_quad,
            Easing::InOutQuad => in_out_quad,

            Easing::InCubic => in_cubic,
            Easing::OutCubic => out_cubic,
            Easing::InOutCubic => in_out_cubic,

            Easing::InQuart => in_quart,
            Easing::OutQuart => out_quart,
            Easing::InOutQuart => in_out_quart,

            Easing::InQuint => in_quint,
            Easing::OutQuint => out_quint,
            Easing::InOutQuint => in_out_quint,

            Easing::InExpo => in_expo,
            Easing::OutExpo => out_expo,
            Easing::InOutExpo => in_out_expo,

            Easing::InCirc => in_circ,
            Easing::OutCirc => out_circ,
            Easing::InOutCirc => in_out_circ,

            Easing::InBack => in_back,
            Easing::OutBack => out_back,
            Easing::InOutBack => in_out_back,

            Easing::InElastic => in_elastic,
            Easing::OutElastic => out_elastic,
            Easing::InOutElastic => in_out_elastic,

            Easing::InBounce => in_bounce,
            Easing::OutBounce => out_bounce,
            Easing::InOutBounce => in_out_bounce,

            Easing::SmoothStep => smooth_step,
            Easing::SmootherStep => smoother_step,
            Easing::StepStart => step_start,
            Easing::StepEnd => step_end,

            Easing::Custom(f) => f,
        };

        EasingRaw { func }
    }

    #[inline(always)]
    pub fn eval(self, t: f32) -> f32 {
        self.to_raw().eval(t)
    }

    #[inline(always)]
    pub fn lerp(self, start: f32, end: f32, t: f32) -> f32 {
        self.to_raw().lerp(start, end, t)
    }
}

impl From<Easing> for EasingRaw {
    #[inline]
    fn from(easing: Easing) -> Self {
        easing.to_raw()
    }
}

#[inline]
pub fn linear(t: f32) -> f32 {
    t
}

#[inline]
pub fn in_sine(t: f32) -> f32 {
    1.0 - ((t * PI) / 2.0).cos()
}

#[inline]
pub fn out_sine(t: f32) -> f32 {
    ((t * PI) / 2.0).sin()
}

#[inline]
pub fn in_out_sine(t: f32) -> f32 {
    -((PI * t).cos() - 1.0) / 2.0
}

#[inline]
pub fn in_quad(t: f32) -> f32 {
    t * t
}

#[inline]
pub fn out_quad(t: f32) -> f32 {
    let u = 1.0 - t;
    1.0 - u * u
}

#[inline]
pub fn in_out_quad(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        let u = -2.0 * t + 2.0;
        1.0 - (u * u) / 2.0
    }
}

#[inline]
pub fn in_cubic(t: f32) -> f32 {
    t * t * t
}

#[inline]
pub fn out_cubic(t: f32) -> f32 {
    let u = 1.0 - t;
    1.0 - u * u * u
}

#[inline]
pub fn in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let u = -2.0 * t + 2.0;
        1.0 - (u * u * u) / 2.0
    }
}

#[inline]
pub fn in_quart(t: f32) -> f32 {
    t * t * t * t
}

#[inline]
pub fn out_quart(t: f32) -> f32 {
    let u = 1.0 - t;
    1.0 - u * u * u * u
}

#[inline]
pub fn in_out_quart(t: f32) -> f32 {
    if t < 0.5 {
        8.0 * t * t * t * t
    } else {
        let u = -2.0 * t + 2.0;
        1.0 - (u * u * u * u) / 2.0
    }
}

#[inline]
pub fn in_quint(t: f32) -> f32 {
    t * t * t * t * t
}

#[inline]
pub fn out_quint(t: f32) -> f32 {
    let u = 1.0 - t;
    1.0 - u * u * u * u * u
}

#[inline]
pub fn in_out_quint(t: f32) -> f32 {
    if t < 0.5 {
        16.0 * t * t * t * t * t
    } else {
        let u = -2.0 * t + 2.0;
        1.0 - (u * u * u * u * u) / 2.0
    }
}

#[inline]
pub fn in_expo(t: f32) -> f32 {
    if t <= 0.0 {
        0.0
    } else {
        (2.0_f32).powf(10.0 * t - 10.0)
    }
}

#[inline]
pub fn out_expo(t: f32) -> f32 {
    if t >= 1.0 {
        1.0
    } else {
        1.0 - (2.0_f32).powf(-10.0 * t)
    }
}

#[inline]
pub fn in_out_expo(t: f32) -> f32 {
    if t <= 0.0 {
        0.0
    } else if t >= 1.0 {
        1.0
    } else if t < 0.5 {
        (2.0_f32).powf(20.0 * t - 10.0) / 2.0
    } else {
        (2.0 - (2.0_f32).powf(-20.0 * t + 10.0)) / 2.0
    }
}

#[inline]
pub fn in_circ(t: f32) -> f32 {
    1.0 - (1.0 - t * t).max(0.0).sqrt()
}

#[inline]
pub fn out_circ(t: f32) -> f32 {
    let u = t - 1.0;
    (1.0 - u * u).max(0.0).sqrt()
}

#[inline]
pub fn in_out_circ(t: f32) -> f32 {
    if t < 0.5 {
        let u = 2.0 * t;
        (1.0 - (1.0 - u * u).max(0.0).sqrt()) / 2.0
    } else {
        let u = -2.0 * t + 2.0;
        ((1.0 - u * u).max(0.0).sqrt() + 1.0) / 2.0
    }
}

const BACK_C1: f32 = 1.70158;
const BACK_C2: f32 = BACK_C1 * 1.525;
const BACK_C3: f32 = BACK_C1 + 1.0;

#[inline]
pub fn in_back(t: f32) -> f32 {
    BACK_C3 * t * t * t - BACK_C1 * t * t
}

#[inline]
pub fn out_back(t: f32) -> f32 {
    let u = t - 1.0;
    1.0 + BACK_C3 * u * u * u + BACK_C1 * u * u
}

#[inline]
pub fn in_out_back(t: f32) -> f32 {
    if t < 0.5 {
        let u = 2.0 * t;
        ((u * u) * ((BACK_C2 + 1.0) * u - BACK_C2)) / 2.0
    } else {
        let u = 2.0 * t - 2.0;
        ((u * u) * ((BACK_C2 + 1.0) * u + BACK_C2) + 2.0) / 2.0
    }
}

const ELASTIC_C4: f32 = (2.0 * PI) / 3.0;
const ELASTIC_C5: f32 = (2.0 * PI) / 4.5;

#[inline]
pub fn in_elastic(t: f32) -> f32 {
    if t <= 0.0 {
        0.0
    } else if t >= 1.0 {
        1.0
    } else {
        -(2.0_f32).powf(10.0 * t - 10.0) * ((t * 10.0 - 10.75) * ELASTIC_C4).sin()
    }
}

#[inline]
pub fn out_elastic(t: f32) -> f32 {
    if t <= 0.0 {
        0.0
    } else if t >= 1.0 {
        1.0
    } else {
        (2.0_f32).powf(-10.0 * t) * ((t * 10.0 - 0.75) * ELASTIC_C4).sin() + 1.0
    }
}

#[inline]
pub fn in_out_elastic(t: f32) -> f32 {
    if t <= 0.0 {
        0.0
    } else if t >= 1.0 {
        1.0
    } else if t < 0.5 {
        -((2.0_f32).powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * ELASTIC_C5).sin()) / 2.0
    } else {
        ((2.0_f32).powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * ELASTIC_C5).sin()) / 2.0 + 1.0
    }
}

#[inline]
pub fn in_bounce(t: f32) -> f32 {
    1.0 - out_bounce(1.0 - t)
}

#[inline]
pub fn out_bounce(mut t: f32) -> f32 {
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;

    if t < 1.0 / D1 {
        N1 * t * t
    } else if t < 2.0 / D1 {
        t -= 1.5 / D1;
        N1 * t * t + 0.75
    } else if t < 2.5 / D1 {
        t -= 2.25 / D1;
        N1 * t * t + 0.9375
    } else {
        t -= 2.625 / D1;
        N1 * t * t + 0.984375
    }
}

#[inline]
pub fn in_out_bounce(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - out_bounce(1.0 - 2.0 * t)) / 2.0
    } else {
        (1.0 + out_bounce(2.0 * t - 1.0)) / 2.0
    }
}

#[inline]
pub fn smooth_step(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

#[inline]
pub fn smoother_step(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline]
pub fn step_start(t: f32) -> f32 {
    if t <= 0.0 { 0.0 } else { 1.0 }
}

#[inline]
pub fn step_end(t: f32) -> f32 {
    if t >= 1.0 { 1.0 } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-4;

    fn assert_near(a: f32, b: f32) {
        assert!((a - b).abs() < EPS, "expected {b}, got {a}");
    }

    #[test]
    fn test_all_easings_endpoints() {
        let easings = [
            Easing::Linear,
            Easing::InSine,
            Easing::OutSine,
            Easing::InOutSine,
            Easing::InQuad,
            Easing::OutQuad,
            Easing::InOutQuad,
            Easing::InCubic,
            Easing::OutCubic,
            Easing::InOutCubic,
            Easing::InQuart,
            Easing::OutQuart,
            Easing::InOutQuart,
            Easing::InQuint,
            Easing::OutQuint,
            Easing::InOutQuint,
            Easing::InExpo,
            Easing::OutExpo,
            Easing::InOutExpo,
            Easing::InCirc,
            Easing::OutCirc,
            Easing::InOutCirc,
            Easing::InBack,
            Easing::OutBack,
            Easing::InOutBack,
            Easing::InElastic,
            Easing::OutElastic,
            Easing::InOutElastic,
            Easing::InBounce,
            Easing::OutBounce,
            Easing::InOutBounce,
            Easing::SmoothStep,
            Easing::SmootherStep,
            Easing::StepStart,
            Easing::StepEnd,
        ];

        for easing in easings {
            let raw: EasingRaw = easing.into();
            assert_near(raw.eval(0.0), 0.0);
            assert_near(raw.eval(1.0), 1.0);
            assert_near(raw.eval(-0.5), 0.0);
            assert_near(raw.eval(1.5), 1.0);
        }
    }

    #[test]
    fn test_inout_midpoint_and_const_raw() {
        const RAW_CUBIC: EasingRaw = Easing::InOutCubic.to_raw();
        assert_near(RAW_CUBIC.eval(0.5), 0.5);
        assert_near(RAW_CUBIC.lerp(10.0, 110.0, 0.5), 60.0);

        let custom = Easing::Custom(|t| t * 0.5).to_raw();
        assert_near(custom.eval(1.0), 0.5);
        assert_near(custom.eval_unclamped(2.0), 1.0);
    }
}
