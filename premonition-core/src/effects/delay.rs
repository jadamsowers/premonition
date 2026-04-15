//! Delay effect with feedback and filtering.

const PI: f32 = 3.14159265359;
const MAX_DELAY_SAMPLES: usize = 88200;

pub struct Delay {
    sample_rate: f32,
    buffer: [f32; MAX_DELAY_SAMPLES],
    write_pos: usize,
    time_ms: f32,
    feedback: f32,
    mix: f32,
    lowpass_state: f32,
    highpass_state: f32,
    filter_coeff: f32,
}

impl Delay {
    pub fn new() -> Self {
        Self {
            sample_rate: 44100.0,
            buffer: [0.0f32; MAX_DELAY_SAMPLES],
            write_pos: 0,
            time_ms: 300.0,
            feedback: 0.3,
            mix: 0.3,
            lowpass_state: 0.0,
            highpass_state: 0.0,
            filter_coeff: 0.8,
        }
    }

    pub fn init(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.buffer = [0.0f32; MAX_DELAY_SAMPLES];
        self.write_pos = 0;
        self.lowpass_state = 0.0;
        self.highpass_state = 0.0;
    }

    pub fn process_mono(&mut self, input: f32) -> f32 {
        self.buffer[self.write_pos] = input + self.lowpass_state * self.feedback;
        self.write_pos = (self.write_pos + 1) % MAX_DELAY_SAMPLES;

        let delay_samples = ((self.time_ms / 1000.0) * self.sample_rate) as usize;
        let delay_samples = delay_samples.min(MAX_DELAY_SAMPLES - 1);

        let read_pos = if self.write_pos >= delay_samples {
            self.write_pos - delay_samples
        } else {
            MAX_DELAY_SAMPLES - (delay_samples - self.write_pos)
        };

        let delayed = self.buffer[read_pos];

        self.lowpass_state =
            self.lowpass_state * self.filter_coeff + delayed * (1.0 - self.filter_coeff);

        input * (1.0 - self.mix) + delayed * self.mix
    }

    pub fn process_stereo(&mut self, input_l: f32, input_r: f32, offset_ms: f32) -> (f32, f32) {
        self.buffer[self.write_pos] =
            (input_l + input_r) * 0.5 + self.lowpass_state * self.feedback;
        self.write_pos = (self.write_pos + 1) % MAX_DELAY_SAMPLES;

        let base_delay = ((self.time_ms / 1000.0) * self.sample_rate) as usize;
        let left_delay = base_delay.min(MAX_DELAY_SAMPLES - 1);
        let right_delay = ((offset_ms / 1000.0) * self.sample_rate) as usize + base_delay;
        let right_delay = right_delay.min(MAX_DELAY_SAMPLES - 1);

        let read_pos_l = if self.write_pos >= left_delay {
            self.write_pos - left_delay
        } else {
            MAX_DELAY_SAMPLES - (left_delay - self.write_pos)
        };

        let read_pos_r = if self.write_pos >= right_delay {
            self.write_pos - right_delay
        } else {
            MAX_DELAY_SAMPLES - (right_delay - self.write_pos)
        };

        let delayed_l = self.buffer[read_pos_l];
        let delayed_r = self.buffer[read_pos_r];

        self.lowpass_state = self.lowpass_state * self.filter_coeff
            + ((delayed_l + delayed_r) * 0.5) * (1.0 - self.filter_coeff);

        (
            input_l * (1.0 - self.mix) + delayed_l * self.mix,
            input_r * (1.0 - self.mix) + delayed_r * self.mix,
        )
    }

    pub fn set_time(&mut self, time_ms: f32) {
        self.time_ms = time_ms.max(1.0).min(2000.0);
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.max(0.0).min(0.95);
    }

    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.max(0.0).min(1.0);
    }

    pub fn set_filter(&mut self, cutoff_hz: f32) {
        self.filter_coeff = (1.0 / (1.0 + (self.sample_rate / (cutoff_hz * 2.0 * PI))))
            .max(0.0)
            .min(1.0);
    }
}

impl Default for Delay {
    fn default() -> Self {
        Self::new()
    }
}
