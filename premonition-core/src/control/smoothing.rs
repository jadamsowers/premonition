//! Parameter smoothing for click-free changes.

#[allow(dead_code)]
pub struct ParameterSmoother {
    current: f32,
    target: f32,
    coefficient: f32,
    enabled: bool,
}

impl ParameterSmoother {
    pub fn new() -> Self {
        Self {
            current: 0.0,
            target: 0.0,
            coefficient: 0.99,
            enabled: true,
        }
    }

    pub fn with_time_constant(sample_rate: f32, time_ms: f32) -> Self {
        let coefficient = (-1.0 / (sample_rate * time_ms / 1000.0)).exp();
        Self {
            current: 0.0,
            target: 0.0,
            coefficient,
            enabled: true,
        }
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    pub fn set_target_immediate(&mut self, target: f32) {
        self.target = target;
        self.current = target;
    }

    pub fn process(&mut self) -> f32 {
        if !self.enabled {
            return self.target;
        }

        self.current = self.current + self.coefficient * (self.target - self.current);
        self.current
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        if !self.enabled {
            return input;
        }
        self.current = self.current + self.coefficient * (input - self.current);
        self.current
    }

    pub fn set_coefficient(&mut self, coeff: f32) {
        self.coefficient = coeff.max(0.0).min(1.0);
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn get_current(&self) -> f32 {
        self.current
    }

    pub fn is_settled(&self) -> bool {
        (self.current - self.target).abs() < 0.0001
    }
}

impl Default for ParameterSmoother {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub struct MultiSmoother {
    smoothers: [ParameterSmoother; 36],
}

impl MultiSmoother {
    pub fn new() -> Self {
        Self {
            smoothers: core::array::from_fn(|_| ParameterSmoother::new()),
        }
    }

    pub fn set_target(&mut self, index: usize, value: f32) {
        if index < self.smoothers.len() {
            self.smoothers[index].set_target(value);
        }
    }

    pub fn process_all(&mut self) -> [f32; 36] {
        let mut output = [0.0f32; 36];
        for (i, smoother) in self.smoothers.iter_mut().enumerate() {
            output[i] = smoother.process();
        }
        output
    }

    pub fn get(&self, index: usize) -> f32 {
        if index < self.smoothers.len() {
            self.smoothers[index].get_current()
        } else {
            0.0
        }
    }
}

impl Default for MultiSmoother {
    fn default() -> Self {
        Self::new()
    }
}
