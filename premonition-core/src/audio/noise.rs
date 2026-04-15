//! Noise generator with pink tilt and per-voice variance.

pub struct Noise {
    voice_index: usize,
    pink_tilt: f32,
    level_variance: f32,
    current_level: f32,
    smoothing: f32,
}

impl Noise {
    pub fn new(voice_index: usize) -> Self {
        let level_variance = ((voice_index as f32 * 7.13).sin() * 0.1).abs();
        Self {
            voice_index,
            pink_tilt: 0.0,
            level_variance,
            current_level: 0.0,
            smoothing: 0.99,
        }
    }

    pub fn init(&mut self, _sample_rate: f32) {
        self.current_level = 0.0;
    }

    pub fn process(&mut self, level: f32) -> f32 {
        let white = Self::generate_white_noise();

        self.pink_tilt = self.pink_tilt * 0.99 + white * 0.01;

        let pink = white * 0.7 + self.pink_tilt * 0.3;

        let target_level = level * (1.0 + self.level_variance * 0.1);
        self.current_level =
            self.current_level * self.smoothing + target_level * (1.0 - self.smoothing);

        let instability = ((self.voice_index as f32 * 13.7).sin() * 0.02).abs() + 1.0;

        pink * self.current_level * instability * 0.5
    }

    fn generate_white_noise() -> f32 {
        let x1 = Self::rand();
        let x2 = Self::rand();
        let x3 = Self::rand();

        ((x1 + x2 + x3) / 3.0) * 2.0 - 1.0
    }

    fn rand() -> f32 {
        static mut STATE: u32 = 12345;
        unsafe {
            STATE = STATE.wrapping_mul(1103515245).wrapping_add(12345);
            ((core::ptr::read_volatile(&raw const STATE) >> 16) as u32 & 0x7FFF) as f32 / 32768.0
        }
    }

    #[allow(dead_code)]
    pub fn set_level_variance(&mut self, variance: f32) {
        self.level_variance = variance;
    }
}

impl Default for Noise {
    fn default() -> Self {
        Self::new(0)
    }
}
