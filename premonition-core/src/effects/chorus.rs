//! BBD-style chorus effect.

const PI: f32 = 3.14159265359;
const MAX_DELAY_SAMPLES: usize = 4096;

pub struct Chorus {
    sample_rate: f32,
    buffer: [f32; MAX_DELAY_SAMPLES],
    write_pos: usize,
    depth: f32,
    rate: f32,
    mix: f32,
    phase: f32,
    feedback: f32,
}

impl Chorus {
    pub fn new() -> Self {
        Self {
            sample_rate: 44100.0,
            buffer: [0.0f32; MAX_DELAY_SAMPLES],
            write_pos: 0,
            depth: 0.003,
            rate: 0.5,
            mix: 0.5,
            phase: 0.0,
            feedback: 0.3,
        }
    }

    pub fn init(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.buffer = [0.0f32; MAX_DELAY_SAMPLES];
        self.write_pos = 0;
        self.phase = 0.0;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        self.buffer[self.write_pos] = input + self.buffer[self.write_pos] * self.feedback;
        self.write_pos = (self.write_pos + 1) % MAX_DELAY_SAMPLES;

        let lfo = (self.phase * 2.0 * PI).sin();
        self.phase += self.rate / self.sample_rate;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        let delay_samples = (self.depth * self.sample_rate * (1.0 + lfo * 0.5)) as usize;
        let delay_samples = delay_samples.min(MAX_DELAY_SAMPLES - 1);

        let read_pos = if self.write_pos >= delay_samples {
            self.write_pos - delay_samples
        } else {
            MAX_DELAY_SAMPLES - (delay_samples - self.write_pos)
        };

        let delayed = self.buffer[read_pos];

        input * (1.0 - self.mix) + delayed * self.mix
    }

    #[allow(dead_code)]
    pub fn process_stereo(&mut self, input_l: f32, input_r: f32) -> (f32, f32) {
        let lfo_l = (self.phase * 2.0 * PI).sin();
        let lfo_r = ((self.phase + 0.5) * 2.0 * PI).sin();

        self.phase += self.rate / self.sample_rate;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        let delayed_l = self.get_delayed_sample(lfo_l);
        let delayed_r = self.get_delayed_sample(lfo_r);

        (
            input_l * (1.0 - self.mix) + delayed_l * self.mix,
            input_r * (1.0 - self.mix) + delayed_r * self.mix,
        )
    }

    #[allow(dead_code)]
    fn get_delayed_sample(&mut self, lfo: f32) -> f32 {
        let delay_samples = (self.depth * self.sample_rate * (1.0 + lfo * 0.5)) as usize;
        let delay_samples = delay_samples.min(MAX_DELAY_SAMPLES - 1);

        let read_pos = if self.write_pos >= delay_samples {
            self.write_pos - delay_samples
        } else {
            MAX_DELAY_SAMPLES - (delay_samples - self.write_pos)
        };

        self.buffer[read_pos]
    }

    #[allow(dead_code)]
    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth.max(0.0).min(0.01);
    }

    #[allow(dead_code)]
    pub fn set_rate(&mut self, rate: f32) {
        self.rate = rate.max(0.1).min(10.0);
    }

    #[allow(dead_code)]
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.max(0.0).min(1.0);
    }

    #[allow(dead_code)]
    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.max(0.0).min(0.9);
    }
}

impl Default for Chorus {
    fn default() -> Self {
        Self::new()
    }
}
