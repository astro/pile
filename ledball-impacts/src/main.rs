extern crate haversine;
extern crate rand;
extern crate ledball;

use std::thread::sleep;
use rand::{thread_rng, Rng, ThreadRng};
use std::time::Duration;
use haversine::{Location, distance, Units};

use ledball::{LedBall, Color};


fn clone_location(l: &Location) -> Location {
    Location {
        latitude: l.latitude,
        longitude: l.longitude,
    }
}

struct Impact {
    pos: Location,
    start: u64,
    color: Color,
}

/// Kilometers per tick
const WAVE_SPEED: f64 = 300.0;
const TICK_INTERVAL: u64 = 50;

impl Impact {
    pub fn is_at(&self, pos: Location) -> u64 {
        let d = distance(clone_location(&self.pos), pos, Units::Kilometers);
        self.start + (d / WAVE_SPEED) as u64
    }
}

struct Rain {
    impacts: Vec<Impact>,
    rng: ThreadRng,
}

impl Rain {
    pub fn new() -> Self {
        Rain {
            impacts: vec![],
            rng: thread_rng(),
        }
    }

    pub fn tick(&mut self, t: u64) {
        if t % TICK_INTERVAL != 0 {
            return
        }

        let pos = Location {
            latitude: self.rng.gen_range(-60.0, 60.0),
            longitude: self.rng.gen_range(-180.0, 180.0),
        };
        // TODO: hsv
        let color = match self.rng.gen_range(0, 6) {
            0 => (255.0, 0.0, 0.0),
            1 => (0.0, 255.0, 0.0),
            2 => (0.0, 0.0, 255.0),
            3 => (255.0, 255.0, 0.0),
            4 => (255.0, 0.0, 255.0),
            5 => (0.0, 255.0, 255.0),
            _ => panic!("Unexpected"),
        };
        self.impacts.push(Impact {
            pos: pos,
            start: t,
            color,
        });

        while self.impacts.len() > 100 {
            self.impacts.remove(0);
        }
    }

    fn get_nearest_at(&self, pos: Location, t: u64) -> Option<&Impact> {
        let mut impacts = self.impacts.iter()
            .map(|impact| (impact, impact.is_at(clone_location(&pos))))
            .filter(|&(ref _impact, impact_time)| impact_time <= t)
            .map(|(impact, _impact_time)| impact)
            .collect::<Vec<&Impact>>();
        impacts.sort_by_key(|&impact| 0 - (impact.start as i64));
        impacts.first().map(|impact| *impact)
    }

    pub fn get_color_at(&self, pos: Location, t: u64) -> Color {
        self.get_nearest_at(pos, t)
            .map(|impact| impact.color.clone())
            .unwrap_or_else(|| (0.0, 0.0, 0.0))
    }
}

pub fn main() {
    let l = LedBall::new("ledball1:2342", 0);
    let mut t = 0u64;
    let mut rain = Rain::new();
    
    loop {
        l.draw(|lat, lon| {
            let pos = Location {
                latitude: lat,
                longitude: lon,
            };
            rain.get_color_at(pos, t)
        });
        t += 1;
        rain.tick(t);

        sleep(Duration::from_millis(20));
    }
}
