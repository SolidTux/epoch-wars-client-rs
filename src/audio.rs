use failure::Error;
use rodio::{default_output_device, Decoder, Device, Sink, Source};
use std::fs::File;
use std::io::BufReader;

pub struct Audio {
    bg_music: String,
}

impl Audio {
    pub fn new() -> Result<Audio, Error> {
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
            bg_music: base_res_path + "res/bg.mp3",
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
        let device = default_output_device().ok_or(format_err!("Unable to open audio device."))?;
        debug!("Playing {}", self.bg_music);
        let bg_file = File::open(&self.bg_music)?;
        let music = ::rodio::play_once(&device, BufReader::new(bg_file))?;
        music.sleep_until_end();
        Ok(())
    }
}
