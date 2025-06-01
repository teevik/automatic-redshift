# Automatic Redshift

A Linux daemon to automatically adjust your screen's color temperature based on your location and time of day.

`GeoClue` and a compositor with `wlr-gamma-control-unstable-v1` support is required.

## How it works

Automatic Redshift automatically:
1. Detects your location using GeoClue
2. Calculates dawn, sunrise, sunset, and dusk times for your location
3. Adjusts your screen color temperature throughout the day:
   - **Day (sunrise to sunset)**: 6500K (cool, blue-rich light)
   - **Night (dusk to dawn)**: 4000K (warm, red-rich light)
   - **Transitions**: Smooth interpolation during dawn and dusk

## Installation

```bash
cargo run
```
