use std::time::SystemTime;

pub struct FrameCounter {
    last_instance: SystemTime,
    last: SystemTime,
    counter: u32,
    pub fps: f32,
}

impl FrameCounter {
    pub fn new() -> Self {
        Self {
            last_instance: SystemTime::now(),
            last: SystemTime::now(),
            counter: 0,
            fps: 0.0,
        }
    }

    pub fn tick(&mut self) -> f32 {
        self.counter += 1;

        let now = SystemTime::now();
        let dur = now.duration_since(self.last).unwrap();
        let dur_in = now.duration_since(self.last_instance).unwrap();
        let elapsed_secs = dur_in.as_secs_f32();

        if elapsed_secs > 1.0 {
            // let elapsed_ms = elapsed_secs * 1000.0;
            // let frame_time = elapsed_ms / self.counter as f32;
            self.fps = self.counter as f32 / elapsed_secs;
            // tracing::info!("Frame time {:.2}ms ({:.1} FPS)", frame_time, self.fps);

            self.last_instance = now;
            self.counter = 0;
        }

        self.last = now;
        dur.as_secs_f32()
    }
}
