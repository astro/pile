extern crate pulse_simple;
extern crate dft;
extern crate palette;

use pulse_simple::Record;
use dft::{Operation, Plan};
use std::thread;
use std::sync::mpsc::{sync_channel, TryRecvError};
use palette::{Hsv, Rgb};
use std::convert::From;
use std::f32::consts::PI;

mod ustripe;
use ustripe::*;

const CHANNELS: usize = 2;
const RATE: u32 = 44100;
const WINDOW: usize = 4096;

const GAMMA: f32 = 2.8;
const MAX_WAVE_AGE: u16 = 100;
const PEAK_CUTOFF: f32 = 0.2;

fn gamma(x: f32) -> u8 {
    (255.0 * (x / 255.0).powf(GAMMA)).min(255.0) as u8
}

fn mix_pixel(dest: &mut [u8; 3], src: &[f32; 3], alpha: f32) {
    for (i, d) in dest.iter_mut().enumerate() {
        *d = ((1.0 - alpha) * *d as f32 + alpha * src[i] as f32) as u8;
    }
}

struct Wave {
    color: [f32; 3],
    age: u16,
    speed: f32
}

struct WaveRenderer {
    waves: Vec<Wave>,
    max: f32,
    base_hue: f32
}

impl WaveRenderer {
    fn new() -> Self {
        WaveRenderer {
            waves: vec![],
            max: 0.0,
            base_hue: 180.0
        }
    }
    fn push_freqs(&mut self, freqs: &[f32]) {
        let mut max = 0.0;
        let mut max_i = 0;
        for (i, val) in freqs.iter().enumerate() {
            if val > &max {
                max = *val;
                max_i = i;
            }
        }

        if max > self.max {
            self.max = max;
        } else {
            self.max *= 0.999;
        }

        if max >= self.max / 4.0 {
            self.base_hue += 1.0;
            while self.base_hue >= 360.0 {
                self.base_hue -= 360.0;
            }

            let color: Rgb = From::from(Hsv::hsv(From::from(self.base_hue - 45.0 * max_i as f32), 1.0, (max / self.max).sqrt()));
            println!("max: {}/{} at {}, hue: {}", max, self.max, max_i, self.base_hue);
            self.waves.push(Wave {
                color: [255.0 * color.red, 255.0 * color.green, 255.0 * color.blue],
                age: 0,
                speed: 2.5 - 2.45 * (0.5 * PI * max_i as f32 / freqs.len() as f32).sin()
            });
        }
    }

    fn render(&mut self, pixels: &mut [[u8; 3]], pitch: i8) {
        // render & advance waves
        for mut wave in self.waves.iter_mut() {
            let position = wave.speed * wave.age as f32;
            let x = if pitch > 0 {
                pitch as i64 * position as i64
            } else {
                pixels.len() as i64 - 1 - -pitch as i64 * position as i64
            };

            for i in 0..4 {
                let x1 = x + i * pitch as i64;
                if x1 > 0 && x1 < pixels.len() as i64 {
                    let a = (1.0 - (wave.age as f32 / MAX_WAVE_AGE as f32) - (i as f32 / 4.0)).max(0.0);  // alpha
                    mix_pixel(&mut pixels[x1 as usize], &wave.color, a);
                }
            }

            wave.age += 1;
        }
        // rm old waves
        self.waves.retain(|wave| wave.age < MAX_WAVE_AGE);
    }
}

struct PeakRenderer {
    value: f32,
    color: [f32; 3],
    max: f32
}

impl PeakRenderer {
    fn new() -> Self {
        PeakRenderer {
            value: 0.0,
            color: [255.0; 3],
            max: 1.0
        }
    }

    fn push_freqs(&mut self, freqs: &[f32]) {
        let mut max = 0.0;
        let mut max_i = 0;
        for (i, val) in freqs.iter().enumerate() {
            if i < (freqs.len() as f32 * PEAK_CUTOFF) as usize && val > &max {
                max = *val;
                max_i = i;
            }
        }

        if max > self.max {
            self.max = max;
        } else {
            self.max -= 4.0;
        }

        if max / self.max >= self.value {
            self.value = (max / self.max).min(1.0);

            self.color = [
                24.0 * max_i as f32,
                16.0 * max_i as f32 * max / self.max,
                255.0 * self.value
            ];
        }
    }

    fn render(&mut self, pixels: &mut [[u8; 3]]) {
        for pixel in pixels.iter_mut() {
            *pixel = [
                gamma(self.value * self.color[2]),
                gamma(self.value * self.color[1]),
                gamma(self.value * self.color[0])
            ];
        }
        self.value = (self.value - 0.05).max(0.0);
    }
}


fn analyze_channel(plan: &Plan, data: &[[f32; CHANNELS]], channel: usize) -> Vec<f32> {
    let mut input = Vec::with_capacity(data.len());
    for x in data {
        input.push(x[channel] as f64);
    }

    dft::transform(&mut input, &plan);
    let output = dft::unpack(&input);

    let mut result = Vec::with_capacity(data.len());
    for ref c in &output[1..(output.len() / 2)] {
        result.push(c.norm() as f32);
    }
    result
}

fn main() {
    let (render_tx, render_rx) = sync_channel::<Option<Vec<Vec<f32>>>>(0);
    thread::spawn(move|| {
        let src = UstripeSource::new("151.216.39.66:2342", 25);
        let mut pixels = Vec::with_capacity(LEDS);
        for _ in 0..LEDS {
            pixels.push([0,0,0]);
        }
        let mut channel_renders = [
            // Left?
            WaveRenderer::new(),
            // Right?
            WaveRenderer::new(),
        ];
        let mut peak_render = PeakRenderer::new();
        let mut paused = false;

        loop {
            match render_rx.try_recv() {
                Ok(None) => paused = true,
                Ok(Some(channel_freqs)) => {
                    for (channel, freqs) in channel_freqs.iter().enumerate() {
                        channel_renders[channel].push_freqs(freqs);
                    }
                    peak_render.push_freqs(&channel_freqs[0]);
                    paused = false;
                },
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => return
            }

            if !paused {
                for pixel in pixels.iter_mut() {
                    *pixel = [0, 0, 0];
                }
                channel_renders[0].render(&mut pixels[20..123], 1);
                channel_renders[1].render(&mut pixels[123..LEDS], -1);
                peak_render.render(&mut pixels[0..20]);
                // for rgb in pixels.iter() {
                //     print!("{:?}", rgb);
                // }
                // println!("");
                src.send(&pixels);
            }
            thread::sleep_ms(20);
        }
    });

    let p = Record::new("ustripe-pulse", "LED-Stripe", RATE);
    let mut plan = Plan::new(Operation::Forward, WINDOW);

    // Fill:
    let mut data = Vec::with_capacity(WINDOW);
    for _ in 0..WINDOW {
        data.push([0.0, 0.0]);
    }

    // Record:
    loop {
        p.read(&mut data[..]);
        if data.iter().all(|pair| pair.iter().all(|c| -0.001 < *c && *c < 0.001)) {
            render_tx.send(None).unwrap();
        } else {
            let mut channel_freqs = Vec::with_capacity(CHANNELS);
            for channel in 0..CHANNELS {
                channel_freqs.push(analyze_channel(&mut plan, &data[..], channel));
            }
            render_tx.send(Some(channel_freqs)).unwrap();
        }
    }
}
