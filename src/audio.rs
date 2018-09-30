use failure::Error;
use rodio::{default_output_device, Decoder, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::Receiver;
use std::thread;

use super::message::AudioMessage;

pub struct Audio {
    bg_music: String,
    build: String,
    rx: Receiver<AudioMessage>,
}

impl Audio {
    pub fn new(rx: Receiver<AudioMessage>) -> Result<Audio, Error> {
        let base_res_path = {
            let mut exe = ::std::env::current_exe().unwrap();
            exe.pop();
            if cfg!(target_os = "macos") {
                exe.to_str().unwrap().to_string() + "/../Resources/"
            } else if cfg!(debug_assertions) {
                String::new()
            } else {
                exe.to_str().unwrap().to_string()
            }
        };
        Ok(Audio {
            bg_music: base_res_path.clone() + "res/bg.ogg",
            build: base_res_path + "res/build.ogg",
            rx,
        })
    }

    pub fn run(&self) {
        if let Err(err) = self.run_res() {
            for e in err.iter_chain() {
                error!("{}", e);
            }
        }
    }

    pub fn run_res(&self) -> Result<(), Error> {
        let bg_music = self.bg_music.clone();
        thread::spawn(move || {
            let device = default_output_device()
                .ok_or(format_err!("Unable to open audio device."))
                .unwrap();
            let sink = Sink::new(&device);
            debug!("Playing {}", bg_music);
            let bg_file = File::open(&bg_music).unwrap();
            let source = Decoder::new(BufReader::new(bg_file)).unwrap();
            sink.append(source.buffered().repeat_infinite());
            sink.sleep_until_end();
        });

        while let Ok(msg) = self.rx.recv() {
            match msg {
                AudioMessage::Build => {
                    let build = self.build.clone();
                    thread::spawn(move || {
                        let device = default_output_device()
                            .ok_or(format_err!("Unable to open audio device."))
                            .unwrap();
                        let sink = Sink::new(&device);
                        debug!("Playing {}", build);
                        let file = File::open(&build).unwrap();
                        let source = Decoder::new(BufReader::new(file)).unwrap();
                        sink.append(source.buffered());
                        sink.sleep_until_end();
                    });
                }
            }
        }
        Ok(())
    }
}
