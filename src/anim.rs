/// 功能：动画插值、过渡状态与时间基准
use crate::ffi::*;

pub fn clock() -> f64 {
    unsafe {
        let mut freq: i64 = 0;
        let mut now: i64 = 0;
        QueryPerformanceFrequency(&mut freq);
        QueryPerformanceCounter(&mut now);
        if freq <= 0 {
            return 0.0;
        }
        now as f64 / freq as f64
    }
}

#[derive(Clone, Copy)]
pub struct Anim {
    pub value: f32,
    target: f32,
    speed: f32,
}

impl Anim {
    pub const fn new(value: f32) -> Anim {
        Anim { value, target: value, speed: 14.0 }
    }

    pub const fn fast(value: f32, speed: f32) -> Anim {
        Anim { value, target: value, speed }
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    pub fn to(&mut self, target: f32) -> bool {
        if (self.target - target).abs() > 0.0001 {
            self.target = target;
            return true;
        }
        self.value != self.target
    }

    pub fn snap(&mut self, value: f32) {
        self.value = value;
        self.target = value;
    }

    pub fn step(&mut self, dt: f32) -> bool {
        let delta = self.target - self.value;
        if delta.abs() < 0.0008 {
            self.value = self.target;
            return false;
        }
        let factor = 1.0 - (-self.speed * dt).exp();
        self.value += delta * factor;
        true
    }
}

#[derive(Clone, Copy)]
pub struct Unused;
