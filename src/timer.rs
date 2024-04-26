pub struct Timer {
    last_render_time: instant::Instant,
    delta_time: instant::Duration
}

impl Timer {
    pub fn new() -> Self {
        let last_render_time = instant::Instant::now();
        let delta_time = instant::Duration::default();

        Self {
            last_render_time,
            delta_time
        }
    }

    pub fn update(&mut self) {
        let now = instant::Instant::now();
        self.delta_time = now - self.last_render_time;
        self.last_render_time = now;
    }

    pub fn delta_time(&self) -> instant::Duration {
        self.delta_time
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

