//! Simple reverb using multiple delay lines (Schroeder reverb style).

const NUM_COMB_FILTERS: usize = 4;
const NUM_ALLPASS_FILTERS: usize = 2;

struct CombFilter {
    buffer: [f32; 2048],
    write_pos: usize,
    feedback: f32,
    damp: f32,
    filter_state: f32,
}

impl CombFilter {
    fn new(_delay_samples: usize, feedback: f32, damp: f32) -> Self {
        Self {
            buffer: [0.0f32; 2048],
            write_pos: 0,
            feedback,
            damp,
            filter_state: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.write_pos];

        self.filter_state = output * (1.0 - self.damp) + self.filter_state * self.damp;
        self.buffer[self.write_pos] = input + self.filter_state * self.feedback;

        self.write_pos = (self.write_pos + 1) & 2047;

        output
    }
}

struct AllpassFilter {
    buffer: [f32; 512],
    write_pos: usize,
    feedback: f32,
}

impl AllpassFilter {
    fn new(_delay_samples: usize, feedback: f32) -> Self {
        Self {
            buffer: [0.0f32; 512],
            write_pos: 0,
            feedback,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.write_pos];
        let output = -input + delayed;

        self.buffer[self.write_pos] = input + delayed * self.feedback;
        self.write_pos = (self.write_pos + 1) & 511;

        output
    }
}

pub struct Reverb {
    sample_rate: f32,
    comb_filters: [CombFilter; NUM_COMB_FILTERS],
    allpass_filters: [AllpassFilter; NUM_ALLPASS_FILTERS],
    wet: f32,
    dry: f32,
    room_size: f32,
    damping: f32,
}

impl Reverb {
    pub fn new() -> Self {
        let comb_delays = [1116, 1188, 1277, 1356];
        let allpass_delays = [556, 441];

        let mut comb_filters = [
            CombFilter::new(comb_delays[0], 0.84, 0.2),
            CombFilter::new(comb_delays[1], 0.84, 0.2),
            CombFilter::new(comb_delays[2], 0.84, 0.2),
            CombFilter::new(comb_delays[3], 0.84, 0.2),
        ];

        for (_i, cf) in comb_filters.iter_mut().enumerate() {
            cf.buffer = [0.0f32; 2048];
            cf.write_pos = 0;
        }

        let allpass_filters = [
            AllpassFilter::new(allpass_delays[0], 0.5),
            AllpassFilter::new(allpass_delays[1], 0.5),
        ];

        Self {
            sample_rate: 44100.0,
            comb_filters,
            allpass_filters,
            wet: 0.3,
            dry: 0.7,
            room_size: 0.5,
            damping: 0.5,
        }
    }

    pub fn init(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;

        for cf in &mut self.comb_filters {
            cf.buffer = [0.0f32; 2048];
            cf.write_pos = 0;
            cf.filter_state = 0.0;
        }

        for af in &mut self.allpass_filters {
            af.buffer = [0.0f32; 512];
            af.write_pos = 0;
        }
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        let input_with_dry = input * self.dry;

        let mut comb_output = 0.0f32;
        for cf in &mut self.comb_filters {
            comb_output += cf.process(input);
        }

        let mut allpass_output = comb_output;
        for af in &mut self.allpass_filters {
            allpass_output = af.process(allpass_output);
        }

        input_with_dry + allpass_output * self.wet
    }

    #[allow(dead_code)]
    pub fn process_stereo(&mut self, input_l: f32, input_r: f32) -> (f32, f32) {
        let mono = (input_l + input_r) * 0.5;
        let reverb_mono = self.process_sample(mono);

        (
            input_l * self.dry + reverb_mono * self.wet,
            input_r * self.dry + reverb_mono * self.wet,
        )
    }

    #[allow(dead_code)]
    pub fn set_wet(&mut self, wet: f32) {
        self.wet = wet.max(0.0).min(1.0);
        self.dry = 1.0 - self.wet * 0.5;
    }

    #[allow(dead_code)]
    pub fn set_room_size(&mut self, size: f32) {
        self.room_size = size.max(0.0).min(1.0);
        let feedback = 0.7 + (self.room_size * 0.28);

        for cf in &mut self.comb_filters {
            cf.feedback = feedback;
        }
    }

    #[allow(dead_code)]
    pub fn set_damping(&mut self, damp: f32) {
        self.damping = damp.max(0.0).min(1.0);

        for cf in &mut self.comb_filters {
            cf.damp = self.damping;
        }
    }
}

impl Default for Reverb {
    fn default() -> Self {
        Self::new()
    }
}
