/// Render a sparkline string from a series of values
/// Uses Unicode block characters: ▁▂▃▄▅▆▇█
pub fn sparkline_string(values: &[f64], width: usize) -> String {
    if values.is_empty() {
        return " ".repeat(width);
    }

    let blocks = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;

    // Sample values to fit width
    let step = values.len() as f64 / width as f64;
    let mut result = String::with_capacity(width);

    for i in 0..width {
        let idx = (i as f64 * step).min((values.len() - 1) as f64) as usize;
        let val = values[idx];

        let level = if range == 0.0 {
            3 // middle
        } else {
            ((val - min) / range * 7.0).round() as usize
        };

        result.push(blocks[level.min(7)]);
    }

    result
}
