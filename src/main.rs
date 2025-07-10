use env_logger::Env;
use futures::{StreamExt, pin_mut};
use geoclue::{Coordinates, location_coordinates_stream};
use jiff::{Timestamp, tz::TimeZone};
use log::{debug, info};
use std::fmt::Display;
use sun::{SunPhase, time_at_phase};
use tokio::{
    select,
    time::{Duration, sleep},
};
use wayland::Wayland;

mod color;
mod geoclue;
mod wayland;

const HIGH_TEMP: u16 = 6500;
const LOW_TEMP: u16 = 4000;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Sun {
    dawn: Timestamp,
    sunrise: Timestamp,
    sunset: Timestamp,
    dusk: Timestamp,
}

fn interpolate_temperature(
    now: Timestamp,
    start: Timestamp,
    stop: Timestamp,
    temp_start: u16,
    temp_stop: u16,
) -> u16 {
    if start == stop {
        return temp_stop;
    }
    let time_pos = (now - start).get_seconds() as f64 / (stop - start).get_seconds() as f64;
    let time_pos = time_pos.clamp(0.0, 1.0);
    let temp_pos = (temp_stop as i32 - temp_start as i32) as f64 * time_pos;
    (temp_start as f64 + temp_pos) as u16
}

fn calculate_sun(now: Timestamp, latitude: f64, longitude: f64) -> Result<Sun, jiff::Error> {
    let now_ms = now.as_millisecond();

    // Try to calculate sun times
    let dawn_ms = time_at_phase(now_ms, SunPhase::Dawn, latitude, longitude, 0.0);
    let sunrise_ms = time_at_phase(now_ms, SunPhase::Sunrise, latitude, longitude, 0.0);
    let sunset_ms = time_at_phase(now_ms, SunPhase::Sunset, latitude, longitude, 0.0);
    let dusk_ms = time_at_phase(now_ms, SunPhase::Dusk, latitude, longitude, 0.0);

    let dawn = Timestamp::from_millisecond(dawn_ms)?;
    let sunrise = Timestamp::from_millisecond(sunrise_ms)?;
    let sunset = Timestamp::from_millisecond(sunset_ms)?;
    let dusk = Timestamp::from_millisecond(dusk_ms)?;

    Ok(Sun {
        dawn,
        sunrise,
        sunset,
        dusk,
    })
}

fn get_temperature(now: Timestamp, sun: Sun) -> u16 {
    if now < sun.dawn {
        LOW_TEMP
    } else if now < sun.sunrise {
        interpolate_temperature(now, sun.dawn, sun.sunrise, LOW_TEMP, HIGH_TEMP)
    } else if now < sun.sunset {
        HIGH_TEMP
    } else if now < sun.dusk {
        interpolate_temperature(now, sun.sunset, sun.dusk, HIGH_TEMP, LOW_TEMP)
    } else {
        LOW_TEMP
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> color_eyre::Result<()> {
    // Panic handler
    color_eyre::install()?;
    let env = Env::default().filter_or("RUST_LOG", "info");
    env_logger::init_from_env(env);

    let mut wayland = Wayland::new()?;

    let mut temp = 6500;

    let location_coordinates_stream = location_coordinates_stream().await?;
    pin_mut!(location_coordinates_stream);

    let mut coordinates = None;
    let mut sun = None;

    loop {
        select! {
            _ = wayland.poll() => (),
            Some(new_coordinates) = location_coordinates_stream.next() => {
                let new_coordinates = new_coordinates?;
                info!("Latitude: {}, Longitude: {}", new_coordinates.latitude, new_coordinates.longitude);
                coordinates = Some(new_coordinates);
            },
            _ = sleep(Duration::from_secs(60)) => (), // Update temperature every minute
        };

        let Some(Coordinates {
            latitude,
            longitude,
        }) = coordinates
        else {
            continue;
        };

        let now = Timestamp::now();
        debug!("Current time: {}", time_of(now));

        let new_sun = calculate_sun(now, latitude, longitude)?;

        if Some(new_sun) != sun {
            sun = Some(new_sun);

            info!(
                "Dawn: {}, Sunrise: {}, Sunset: {}, Dusk: {}",
                time_of(new_sun.dawn),
                time_of(new_sun.sunrise),
                time_of(new_sun.sunset),
                time_of(new_sun.dusk)
            );
        }

        let new_temp = get_temperature(now, new_sun);

        debug!("Calculated temperature: {new_temp} K");

        if new_temp != temp {
            temp = new_temp;
            wayland.set_temperature(temp)?;
            info!("Updated temperature to {} K", temp);
        } else {
            debug!("Temperature unchanged at {} K", temp);
        }
    }
}

/// Format time as HH:MM
fn time_of(timestamp: Timestamp) -> impl Display {
    let timezone = TimeZone::system();
    let zoned = timestamp.to_zoned(timezone);

    zoned.time().strftime("%H:%M")
}
