use rodio::Sink;

pub struct Audio {
    sink: Sink,
}

impl Audio {
    pub fn new() -> Self {
        let audio_source = rodio::source::SineWave::new(1024);
        let sink = Sink::new(&rodio::default_output_device().unwrap());
        sink.pause();
        sink.append(audio_source);
        Self { sink }
    }

    pub fn play(&self) {
        self.sink.play()
    }

    pub fn pause(&self) {
        self.sink.pause()
    }
}
