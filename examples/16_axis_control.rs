//! Axis control demo: xlim/ylim, custom ticks, tick rotation, and grid styling.

use plotkit::prelude::*;
use std::f64::consts::{FRAC_PI_2, PI, TAU};

fn main() -> plotkit::Result<()> {
    // Generate sine data.
    let x: Vec<f64> = (0..200).map(|i| i as f64 * 0.05).collect();
    let y: Vec<f64> = x.iter().map(|&v| v.sin()).collect();

    let mut fig = Figure::with_size(900, 700);

    // --- Subplot 1: xlim/ylim with custom ticks ---
    {
        let ax = fig.add_subplot(2, 2, 1);
        ax.plot(&x, &y)?.label("sin(x)");
        ax.set_title("xlim + ylim + custom ticks");
        ax.set_xlim(0.0, TAU);
        ax.set_ylim(-1.2, 1.2);
        ax.set_xticks(&[0.0, FRAC_PI_2, PI, 3.0 * FRAC_PI_2, TAU]);
        ax.set_xticklabels(&["0", "pi/2", "pi", "3pi/2", "2pi"]);
        ax.set_yticks(&[-1.0, -0.5, 0.0, 0.5, 1.0]);
    }

    // --- Subplot 2: Inverted axes ---
    {
        let ax = fig.add_subplot(2, 2, 2);
        ax.plot(&x, &y)?;
        ax.set_title("Inverted x-axis");
        ax.set_xlim(0.0, TAU);
        ax.invert_xaxis();
    }

    // --- Subplot 3: Tick rotation ---
    {
        let ax = fig.add_subplot(2, 2, 3);
        ax.plot(&x, &y)?;
        ax.set_title("Rotated tick labels (45 deg)");
        ax.set_xlim(0.0, 10.0);
        ax.tick_params_x_rotation(45.0);
    }

    // --- Subplot 4: Grid control ---
    {
        let ax = fig.add_subplot(2, 2, 4);
        ax.plot(&x, &y)?;
        ax.set_title("Dashed grid, y-only, alpha=0.4");
        ax.grid(true);
        ax.grid_axis("y");
        ax.grid_alpha(0.4);
        ax.grid_style(LineStyle::Dashed);
    }

    fig.save("examples/output/16_axis_control.png")?;
    Ok(())
}
