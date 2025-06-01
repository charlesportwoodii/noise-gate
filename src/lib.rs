//!
//!
//!
//!
//!
//!
//!
//!
//!

/// The Noise Gate
/// This should be implemented outside of your main audio loop so that the open/close thresholds, and other settings can persist across the stream
pub struct NoiseGate {
    /// The open threshold as db (eg -36.0)
    open_threshold: f32,
    /// The close threshold as db (eg -56.0)
    close_threshold: f32,
    /// The sample rate in hz (eg 48000.0)
    sample_rate: f32,
    /// The relesae rate, in ms (eg 150)
    release_rate: f32,
    /// The attack rate in ms
    attack_rate: f32,
    decay_rate: f32,
    /// How long the gate should be held open for in ms
    hold_time: f32,
    /// The number of audio channels in your stream
    channels: usize,
    is_open: bool,
    attenuation: f32,
    level: f32,
    held_time: f32,
}

impl NoiseGate {
    /// Create a new noise gate.
    pub fn new(
        open_threshold: f32,
        close_threshold: f32,
        sample_rate: f32,
        channels: usize,
        release_rate: f32,
        attack_rate: f32,
        hold_time: f32
    ) -> Self {
        let threshold_diff = open_threshold - close_threshold;
        let min_decay_period = (1.0 / 75.0) * sample_rate;

        Self {
            open_threshold: match open_threshold.is_finite() {
                true => (10_f32).powf(open_threshold / 20.0),
                false => 0.0,
            },
            close_threshold: match close_threshold.is_finite() {
                true => (10_f32).powf(close_threshold / 20.0),
                false => 0.0,
            },
            sample_rate: 1.0 / sample_rate,
            channels: channels,
            release_rate: 1.0 / (release_rate * 0.001 * sample_rate),
            attack_rate: 1.0 / (attack_rate * 0.001 * sample_rate),
            decay_rate: threshold_diff / min_decay_period,
            hold_time: hold_time * 0.001,
            is_open: false,
            attenuation: 0.0,
            level: 0.0,
            held_time: 0.0,
        }
    }

    pub fn update(
        &mut self,
        open_threshold: f32,
        close_threshold: f32,
        release_rate: f32,
        attack_rate: f32,
        hold_time: f32
    ) {
        let threshold_diff = open_threshold - close_threshold;
        let min_decay_period = (1.0 / 75.0) * self.sample_rate;

        self.open_threshold = match open_threshold.is_finite() {
            true => (10_f32).powf(open_threshold / 20.0),
            false => 0.0,
        };
        self.close_threshold = match close_threshold.is_finite() {
            true => (10_f32).powf(close_threshold / 20.0),
            false => 0.0,
        };
        self.release_rate = 1.0 / (release_rate * 0.001 * self.sample_rate);
        self.attack_rate = 1.0 / (attack_rate * 0.001 * self.sample_rate);
        self.decay_rate = threshold_diff / min_decay_period;
        self.hold_time = hold_time * 0.001;
    }

    /// Takes a frame and returns a new frame that has been attenuated by the gate
    pub fn process_frame(&mut self, frame: &[f32]) -> Vec<f32> {
        let mut channel_frames = Vec::<Vec<f32>>::new();
        for _ in 0..self.channels {
            channel_frames.push(Vec::<f32>::with_capacity(frame.len() / self.channels));
        }

        for c in 0..self.channels {
            for (_, u) in frame.iter().enumerate().skip(c).step_by(self.channels) {
                channel_frames[c].push(*u);
            }
        }

        let mut resample = Vec::<f32>::with_capacity(frame.len());

        for i in 0..channel_frames[0].len() {
            let mut current_level = f32::abs(channel_frames[0][i]);

            for j in 0..self.channels {
                current_level = f32::max(current_level, channel_frames[j][i]);
            }

            if current_level > self.open_threshold && !self.is_open {
                self.is_open = true;
            }

            if self.level < self.close_threshold && self.is_open {
                self.held_time = 0.0;
                self.is_open = false;
            }

            self.level = f32::max(self.level, current_level) - self.decay_rate;

            if self.is_open {
                self.attenuation = f32::min(1.0, self.attenuation + self.attack_rate);
            } else {
                self.held_time += self.sample_rate;
                if self.held_time > self.hold_time {
                    self.attenuation = f32::max(0.0, self.attenuation - self.release_rate);
                }
            }

            for c in 0..self.channels {
                channel_frames[c][i] *= self.attenuation;
            }
        }

        // We need to flatten this back down to a single vec
        // For each channel
        // Grab the next element and push it to resample
        for i in 0..channel_frames[0].len() {
            for c in 0..self.channels {
                resample.push(channel_frames[c][i]);
            }
        }

        return resample.into();
    }
}
