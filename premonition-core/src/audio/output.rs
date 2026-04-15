//! Output stage with voice summing and soft-clipping.

pub struct OutputStage {
    num_voices: usize,
    master_volume: f32,
    soft_clip_threshold: f32,
    noise_floor: f32,
    asymmetry: f32,
    dc_offset_l: f32,
    dc_offset_r: f32,
}

impl OutputStage {
    pub fn new(num_voices: usize) -> Self {
        Self {
            num_voices,
            master_volume: 0.75,
            soft_clip_threshold: 0.9,
            noise_floor: 0.0001,
            asymmetry: 0.01,
            dc_offset_l: 0.0,
            dc_offset_r: 0.0,
        }
    }

    pub fn init(&mut self, _sample_rate: f32) {
        self.dc_offset_l = (Self::rand_simple() * 0.002) - 0.001;
        self.dc_offset_r = (Self::rand_simple() * 0.002) - 0.001;
    }

    #[allow(dead_code)]
    pub fn process(&mut self, voice_l: f32, voice_r: f32, master_vol: f32) -> (f32, f32) {
        self.master_volume = master_vol;

        let mixed_l = voice_l;
        let mixed_r = voice_r;

        let clipped_l = self.soft_clip(mixed_l);
        let clipped_r = self.soft_clip(mixed_r);

        let output_l = clipped_l * self.master_volume;
        let output_r = clipped_r * self.master_volume;

        let output_l_with_dc = output_l + self.dc_offset_l;
        let output_r_with_dc = output_r + self.dc_offset_r;

        let output_l_noisy = output_l_with_dc + (Self::rand_simple() - 0.5) * self.noise_floor;
        let output_r_noisy = output_r_with_dc + (Self::rand_simple() - 0.5) * self.noise_floor;

        (output_l_noisy, output_r_noisy)
    }

    #[allow(dead_code)]
    fn soft_clip(&self, input: f32) -> f32 {
        let positive_asymmetry = self.soft_clip_positive(input);
        let negative_asymmetry = self.soft_clip_negative(input);
        positive_asymmetry + negative_asymmetry - input
    }

    #[allow(dead_code)]
    fn soft_clip_positive(&self, input: f32) -> f32 {
        if input > 0.0 {
            input / (1.0 + (input * input * self.asymmetry))
        } else {
            0.0
        }
    }

    #[allow(dead_code)]
    fn soft_clip_negative(&self, input: f32) -> f32 {
        if input < 0.0 {
            input / (1.0 + (input * input * self.asymmetry))
        } else {
            0.0
        }
    }

    fn rand_simple() -> f32 {
        let result;
        unsafe {
            let x = core::ptr::read_volatile(&raw const OUTPUT_STATE);
            OUTPUT_STATE = x.wrapping_mul(1103515245).wrapping_add(12345);
            result = ((OUTPUT_STATE >> 16) as u32 & 0x7FFF) as f32 / 32768.0;
        }
        result
    }

    #[allow(dead_code)]
    pub fn apply_stereo_spread(&self, input: f32, pan_position: f32) -> (f32, f32) {
        let left_gain = (1.0 - pan_position).sqrt();
        let right_gain = (1.0 + pan_position).sqrt();
        (input * left_gain, input * right_gain)
    }

    #[allow(dead_code)]
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.max(0.0).min(1.0);
    }

    #[allow(dead_code)]
    pub fn set_noise_floor(&mut self, noise: f32) {
        self.noise_floor = noise.max(0.0);
    }
}

static mut OUTPUT_STATE: u32 = 54321;

impl Default for OutputStage {
    fn default() -> Self {
        Self::new(8)
    }
}
