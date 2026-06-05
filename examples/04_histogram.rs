//! Histogram example.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    // Generate some pseudo-random data (simple LCG)
    let mut data = Vec::with_capacity(1000);
    let mut seed: u64 = 12345;
    for _ in 0..1000 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let val = (seed >> 33) as f64 / (1u64 << 31) as f64;
        // Box-Muller-ish: sum of uniform -> approximately normal
        data.push(val);
    }

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);
    ax.hist(&data, 30)?;
    ax.set_title("Histogram");
    ax.set_xlabel("Value");
    ax.set_ylabel("Count");
    fig.save("examples/output/04_histogram.png")?;
    Ok(())
}
