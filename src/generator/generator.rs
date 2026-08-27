use crate::config::config::AppConfig;
use crate::dto::request::audio::AudioData;
use crate::services::socket::SocketService;

use crate::dto::request::frame::FrameData;
use anyhow::Result;
use env_logger::Builder;
use image::codecs::jpeg::JpegEncoder;
use image::{ImageBuffer, RgbImage};
use log::{error, info};
use rand::Rng;
use std::thread;
use std::time::Duration;

const SPEECH_DURATION_RANGE: std::ops::RangeInclusive<f32> = 1.0..=3.0;
const SILENCE_DURATION_RANGE: std::ops::RangeInclusive<f32> = 0.5..=2.0;

pub fn run(cfg: &AppConfig) -> Result<()> {
    Builder::new().filter_level(cfg.log_level()).init();

    let audio_addr = cfg.audio_addr();
    let frame_addr = cfg.frame_addr();

    let fft_size = cfg.audio.fft_size;
    let sample_rate = cfg.audio.sample_rate;
    let audio_send_buf = cfg.send_buf().clone();
    let frame_send_buf = cfg.send_buf().clone();
    let frame_width = cfg.frame.width;
    let frame_height = cfg.frame.height;

    let audio_client = SocketService::bind("127.0.0.1:0")?;
    let frame_client = SocketService::bind("127.0.0.1:0")?;

    thread::spawn(move || {
        let packet_duration = fft_size as f32 / sample_rate as f32;
        let mut rng = rand::thread_rng();
        let mut speech: bool = rand::random();
        let mut remaining_time = if speech {
            rng.gen_range(SPEECH_DURATION_RANGE)
        } else {
            rng.gen_range(SILENCE_DURATION_RANGE)
        };
        let mut send_buf = audio_send_buf;

        loop {
            let audio = create_audio(fft_size, sample_rate, speech);
            send_audio(&audio_addr, audio, &audio_client, &mut send_buf);

            remaining_time -= packet_duration;
            if remaining_time <= 0.0 {
                speech = rand::random();
                remaining_time = if speech {
                    rng.gen_range(SPEECH_DURATION_RANGE)
                } else {
                    rng.gen_range(SILENCE_DURATION_RANGE)
                };
                info!(
                    "[АУДИО] Переключение на {}",
                    if speech { "речь" } else { "тишину" }
                );
            }
            thread::sleep(Duration::from_millis(10));
        }
    });

    thread::spawn(move || {
        let mut rng = rand::thread_rng();
        let mut send_buf = frame_send_buf;

        loop {
            let frame = generate_frame(frame_width, frame_height, &mut rng);
            send_frame(&frame_addr, frame, &frame_client, &mut send_buf);
            thread::sleep(Duration::from_millis(10));
        }
    });

    loop {}
}

fn create_audio(fft_size: usize, sample_rate: u32, speech: bool) -> AudioData {
    if !speech {
        return AudioData {
            mic1: vec![0; fft_size],
            mic2: vec![0; fft_size],
        };
    }

    let mut rng = rand::thread_rng();

    let freq_hz = rng.gen_range(100.0..=2000.0);
    let amplitude = rng.gen_range(5000.0..=30000.0);
    let delay_sec = rng.gen_range(0.0..=0.001);

    info!(
        "Генерация аудио: freq={:.1} Гц, амплитуда={:.1}, задержка={:.6} с",
        freq_hz, amplitude, delay_sec
    );

    let mic1: Vec<i16> = (0..fft_size)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            (amplitude * (2.0 * std::f32::consts::PI * freq_hz * t).sin()) as i16
        })
        .collect();

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

fn generate_frame(width: u32, height: u32, rng: &mut impl Rng) -> FrameData {
    let mut img: RgbImage = ImageBuffer::new(width, height);
    for pixel in img.pixels_mut() {
        let r = rng.r#gen::<u8>();
        let g = rng.r#gen::<u8>();
        let b = rng.r#gen::<u8>();
        *pixel = image::Rgb([r, g, b]);
    }

    let mut frame = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut frame, 45);
    encoder
        .encode(&img, width, height, image::ColorType::Rgb8)
        .unwrap();

    info!("Генерация кадра: {} x {}", width, height);

    FrameData { frame }
}

fn send_audio(addr_str: &str, audio: AudioData, client: &SocketService, send_buf: &mut Vec<u8>) {
    let addr = match addr_str.parse() {
        Ok(a) => a,
        Err(e) => {
            error!("Ошибка парсинга адреса '{}': {}", addr_str, e);
            return;
        }
    };

    if let Err(e) = client.send_to(&audio, addr, send_buf) {
        error!("[АУДИО] Ошибка отправки: {}", e);
    }
}

fn send_frame(addr_str: &str, frame: FrameData, client: &SocketService, send_buf: &mut Vec<u8>) {
    let addr = match addr_str.parse() {
        Ok(a) => a,
        Err(e) => {
            error!("Ошибка парсинга адреса '{}': {}", addr_str, e);
            return;
        }
    };

    if let Err(e) = client.send_to(&frame, addr, send_buf) {
        error!("[КАДР] Ошибка отправки: {}", e);
    }
}
