//! Hexagonal binning (hexbin) example with 2D Gaussian scatter data.

use plotkit::prelude::*;

fn main() -> plotkit::Result<()> {
    // Generate 2D Gaussian data using a simple Box-Muller transform.
    let n = 10_000;
    let mut x = Vec::with_capacity(n);
    let mut y = Vec::with_capacity(n);

    // Simple pseudo-random number generator (xorshift64).
    let mut state: u64 = 0xDEAD_BEEF_CAFE_BABE;
    let mut rand_f64 = || -> f64 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state as f64) / (u64::MAX as f64)
    };

    for _ in 0..n {
        // Box-Muller transform for normal distribution.
        let u1 = rand_f64().max(1e-15);
        let u2 = rand_f64();
        let r = (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * std::f64::consts::PI * u2;
        x.push(r * theta.cos());
        y.push(r * theta.sin());
    }

    let mut fig = Figure::with_size(800, 600);
    let ax = fig.add_subplot(1, 1, 1);

    ax.hexbin(x, y)?
        .gridsize(25)
        .colormap(Colormap::Viridis)
        .mincnt(1)
        .edgecolor(Color::rgb(40, 40, 40))
        .alpha(0.9)
        .colorbar(true)
        .label("density");

    ax.set_title("Hexagonal Binning of 2D Gaussian");
    ax.set_xlabel("x");
    ax.set_ylabel("y");

    fig.save("examples/output/20_hexbin.png")?;
    Ok(())
}
