use crate::config::config::AppConfig;
use crate::dto::request::audio::AudioData;
use crate::services::socket::SocketService;

use anyhow::Result;
use env_logger::Builder;
use log::{error, info};
use rand::Rng;
use std::thread;
use std::time::Duration;

pub fn run(cfg: &AppConfig) -> Result<()> {
    Builder::new().filter_level(cfg.log_level()).init();

    let addr = cfg.addr().parse()?;
    let fft_size = cfg.vad.fft_size;
    let sample_rate = cfg.audio.sample_rate;
    let send_buf = cfg.send_buf();

    thread::spawn(move || {
        let client = SocketService::bind("127.0.0.1:0").unwrap();
        let packet_duration = fft_size as f32 / sample_rate as f32;

        let mut rng = rand::thread_rng();
        let mut speech: bool = rand::random();

        let mut remaining_time = if speech {
            rng.gen_range(1.0..=3.0)
        } else {
            rng.gen_range(0.5..=2.0)
        };

        let mut send_buf = send_buf;

        loop {
            let audio = create_audio(fft_size, sample_rate, speech);
            if let Err(e) = client.send_to(&audio, addr, &mut send_buf) {
                error!("Ошибка отправки: {}", e);
            }

            remaining_time -= packet_duration;
            if remaining_time <= 0.0 {

                speech = rand::random();
                remaining_time = if speech {
                    rng.gen_range(1.0..=3.0)
                } else {
                    rng.gen_range(0.5..=2.0)
                };
                info!("Переключение на {}", if speech { "речь" } else { "тишину" });
            }

            thread::sleep(Duration::from_millis(10));
        }
    });

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

fn create_audio(fft_size: usize, sample_rate: u32, speech: bool) -> AudioData {
    if !speech {
        return AudioData {
            mic1: vec![0; fft_size],
            mic2: vec![0; fft_size],
        };
    }

    let freq_hz = 500.0;
    let amplitude = 20000.0;
    let mic1: Vec<i16> = (0..fft_size)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            (amplitude * (2.0 * std::f32::consts::PI * freq_hz * t).sin()) as i16
        })
        .collect();
    let delay_sec = 0.0001;
    let mic2: Vec<i16> = (0..fft_size)
        .map(|i| {
            let t = i as f32 / sample_rate as f32 - delay_sec;
            if t >= 0.0 {
                (amplitude * (2.0 * std::f32::consts::PI * freq_hz * t).sin()) as i16
            } else {
                0
            }
        })
        .collect();

    AudioData { mic1, mic2 }
}
