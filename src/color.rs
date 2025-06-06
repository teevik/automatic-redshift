use color_eyre::eyre::bail;
use vek::Rgb;

/// Fill a color ramp based on a temperature
pub fn fill_colorramp(
    r: &mut [u16],
    g: &mut [u16],
    b: &mut [u16],
    ramp_size: usize,
    temperature: u16,
) -> color_eyre::Result<()> {
    let temperature_color = find_interpolated_temperature_color(temperature as f64)?;

    let max_intensity = u16::MAX as f64;
    let step = max_intensity / (ramp_size - 1) as f64;

    for i in 0..ramp_size {
        let intensity = step * i as f64;

        r[i] = (intensity * temperature_color.r) as u16;
        g[i] = (intensity * temperature_color.g) as u16;
        b[i] = (intensity * temperature_color.b) as u16;
    }

    Ok(())
}

fn find_interpolated_temperature_color(temperature: f64) -> color_eyre::Result<Rgb<f64>> {
    if temperature < 1000.0 || temperature > 10000.0 {
        bail!("Temperature must be between 1000 and 10000");
    }

    let current = (temperature - 1000.) as usize / 100;
    let next = current + 1;
    let factor = temperature % 100.;

    let color = Rgb::lerp(
        COLOR_FOR_TEMPERATURE[current],
        COLOR_FOR_TEMPERATURE[next],
        factor,
    );

    Ok(color)
}

/// [Black body radiation color](https://en.wikipedia.org/wiki/Black-body_radiation) mapped by
/// temperatures in the range `1000.0..=10100.0` with a step of 100.0
///
/// Refer to <https://gitlab.com/chinstrap/gammastep/-/blob/master/README-colorramp> for more info.
const COLOR_FOR_TEMPERATURE: [Rgb<f64>; 92] = [
    Rgb::new(1.00000000, 0.18172716, 0.00000000),
    Rgb::new(1.00000000, 0.25503671, 0.00000000),
    Rgb::new(1.00000000, 0.30942099, 0.00000000),
    Rgb::new(1.00000000, 0.35357379, 0.00000000),
    Rgb::new(1.00000000, 0.39091524, 0.00000000),
    Rgb::new(1.00000000, 0.42322816, 0.00000000),
    Rgb::new(1.00000000, 0.45159884, 0.00000000),
    Rgb::new(1.00000000, 0.47675916, 0.00000000),
    Rgb::new(1.00000000, 0.49923747, 0.00000000),
    Rgb::new(1.00000000, 0.51943421, 0.00000000),
    Rgb::new(1.00000000, 0.54360078, 0.08679949),
    Rgb::new(1.00000000, 0.56618736, 0.14065513),
    Rgb::new(1.00000000, 0.58734976, 0.18362641),
    Rgb::new(1.00000000, 0.60724493, 0.22137978),
    Rgb::new(1.00000000, 0.62600248, 0.25591950),
    Rgb::new(1.00000000, 0.64373109, 0.28819679),
    Rgb::new(1.00000000, 0.66052319, 0.31873863),
    Rgb::new(1.00000000, 0.67645822, 0.34786758),
    Rgb::new(1.00000000, 0.69160518, 0.37579588),
    Rgb::new(1.00000000, 0.70602449, 0.40267128),
    Rgb::new(1.00000000, 0.71976951, 0.42860152),
    Rgb::new(1.00000000, 0.73288760, 0.45366838),
    Rgb::new(1.00000000, 0.74542112, 0.47793608),
    Rgb::new(1.00000000, 0.75740814, 0.50145662),
    Rgb::new(1.00000000, 0.76888303, 0.52427322),
    Rgb::new(1.00000000, 0.77987699, 0.54642268),
    Rgb::new(1.00000000, 0.79041843, 0.56793692),
    Rgb::new(1.00000000, 0.80053332, 0.58884417),
    Rgb::new(1.00000000, 0.81024551, 0.60916971),
    Rgb::new(1.00000000, 0.81957693, 0.62893653),
    Rgb::new(1.00000000, 0.82854786, 0.64816570),
    Rgb::new(1.00000000, 0.83717703, 0.66687674),
    Rgb::new(1.00000000, 0.84548188, 0.68508786),
    Rgb::new(1.00000000, 0.85347859, 0.70281616),
    Rgb::new(1.00000000, 0.86118227, 0.72007777),
    Rgb::new(1.00000000, 0.86860704, 0.73688797),
    Rgb::new(1.00000000, 0.87576611, 0.75326132),
    Rgb::new(1.00000000, 0.88267187, 0.76921169),
    Rgb::new(1.00000000, 0.88933596, 0.78475236),
    Rgb::new(1.00000000, 0.89576933, 0.79989606),
    Rgb::new(1.00000000, 0.90198230, 0.81465502),
    Rgb::new(1.00000000, 0.90963069, 0.82838210),
    Rgb::new(1.00000000, 0.91710889, 0.84190889),
    Rgb::new(1.00000000, 0.92441842, 0.85523742),
    Rgb::new(1.00000000, 0.93156127, 0.86836903),
    Rgb::new(1.00000000, 0.93853986, 0.88130458),
    Rgb::new(1.00000000, 0.94535695, 0.89404470),
    Rgb::new(1.00000000, 0.95201559, 0.90658983),
    Rgb::new(1.00000000, 0.95851906, 0.91894041),
    Rgb::new(1.00000000, 0.96487079, 0.93109690),
    Rgb::new(1.00000000, 0.97107439, 0.94305985),
    Rgb::new(1.00000000, 0.97713351, 0.95482993),
    Rgb::new(1.00000000, 0.98305189, 0.96640795),
    Rgb::new(1.00000000, 0.98883326, 0.97779486),
    Rgb::new(1.00000000, 0.99448139, 0.98899179),
    Rgb::new(1.00000000, 1.00000000, 1.00000000),
    Rgb::new(0.98947904, 0.99348723, 1.00000000),
    Rgb::new(0.97940448, 0.98722715, 1.00000000),
    Rgb::new(0.96975025, 0.98120637, 1.00000000),
    Rgb::new(0.96049223, 0.97541240, 1.00000000),
    Rgb::new(0.95160805, 0.96983355, 1.00000000),
    Rgb::new(0.94303638, 0.96443333, 1.00000000),
    Rgb::new(0.93480451, 0.95923080, 1.00000000),
    Rgb::new(0.92689056, 0.95421394, 1.00000000),
    Rgb::new(0.91927697, 0.94937330, 1.00000000),
    Rgb::new(0.91194747, 0.94470005, 1.00000000),
    Rgb::new(0.90488690, 0.94018594, 1.00000000),
    Rgb::new(0.89808115, 0.93582323, 1.00000000),
    Rgb::new(0.89151710, 0.93160469, 1.00000000),
    Rgb::new(0.88518247, 0.92752354, 1.00000000),
    Rgb::new(0.87906581, 0.92357340, 1.00000000),
    Rgb::new(0.87315640, 0.91974827, 1.00000000),
    Rgb::new(0.86744421, 0.91604254, 1.00000000),
    Rgb::new(0.86191983, 0.91245088, 1.00000000),
    Rgb::new(0.85657444, 0.90896831, 1.00000000),
    Rgb::new(0.85139976, 0.90559011, 1.00000000),
    Rgb::new(0.84638799, 0.90231183, 1.00000000),
    Rgb::new(0.84153180, 0.89912926, 1.00000000),
    Rgb::new(0.83682430, 0.89603843, 1.00000000),
    Rgb::new(0.83225897, 0.89303558, 1.00000000),
    Rgb::new(0.82782969, 0.89011714, 1.00000000),
    Rgb::new(0.82353066, 0.88727974, 1.00000000),
    Rgb::new(0.81935641, 0.88452017, 1.00000000),
    Rgb::new(0.81530175, 0.88183541, 1.00000000),
    Rgb::new(0.81136180, 0.87922257, 1.00000000),
    Rgb::new(0.80753191, 0.87667891, 1.00000000),
    Rgb::new(0.80380769, 0.87420182, 1.00000000),
    Rgb::new(0.80018497, 0.87178882, 1.00000000),
    Rgb::new(0.79665980, 0.86943756, 1.00000000),
    Rgb::new(0.79322843, 0.86714579, 1.00000000),
    Rgb::new(0.78988728, 0.86491137, 1.00000000),
    Rgb::new(0.78663296, 0.86273225, 1.00000000),
];
