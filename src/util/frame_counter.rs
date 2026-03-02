const MAXIMUM_SAMPLES: usize = 100;

pub struct FrameCounter {
    pub total_frames: u64,
    pub total_seconds: f32,
    pub avg_fps: f32,
    pub cur_fps: f32,

    sample_buffer: Vec<f32>
}
impl FrameCounter {
    pub fn new() -> Self {
        Self {
            total_frames: 0,
            total_seconds: 0.0,
            avg_fps: 0.0,
            cur_fps: 0.0,

            sample_buffer: Vec::new()
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.cur_fps = 1.0 / delta_time;
        if !self.cur_fps.is_infinite() {
            self.sample_buffer.push(self.cur_fps);
        }

        if self.sample_buffer.len() > MAXIMUM_SAMPLES {
            self.sample_buffer.remove(0);
            self.avg_fps = self.sample_buffer.iter().sum::<f32>() / self.sample_buffer.len() as f32;
        } else {
            self.avg_fps = self.cur_fps;
        }

        self.total_frames += 1;
        self.total_seconds += delta_time;
    }
}