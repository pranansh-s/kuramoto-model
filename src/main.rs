mod sim;
#[cfg(feature = "viz")]
mod viz;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "Kuramoto model simulation", long_about = None)]
struct Args {
    /// Number of oscillators
    #[arg(short, long, default_value_t = 200)]
    n_oscillators: usize,

    /// Run without visualiser (K-sweep → CSV)
    #[arg(long, default_value_t = false)]
    headless: bool,

    /// Time step dt
    #[arg(long, default_value_t = 0.01)]
    dt: f64,

    /// Number of time steps per K value
    #[arg(short, long, default_value_t = 5000)]
    steps: usize,

    /// Number of K values to sweep
    #[arg(long, default_value_t = 100)]
    k_steps: usize,

    /// Maximum coupling strength K
    #[arg(long, default_value_t = 5.0)]
    k_max: f64,

    /// Output CSV path
    #[arg(short, long, default_value = "data.csv")]
    output: PathBuf,

    // ── visual-mode option ──
    /// Initial coupling strength for visual mode
    #[arg(short, long, default_value_t = 0.5)]
    k: f64,
}

fn main() {
    let args = Args::parse();

    if args.headless {
        run_headless(&args);
    } else {
        #[cfg(feature = "viz")]
        {
            viz::run(args.n_oscillators, args.k as f32, args.dt);
        }
    }
}

fn run_headless(args: &Args) {
    let mut writer = csv::Writer::from_path(&args.output).expect("Cannot create output file");
    writer.write_record(&["K", "r"]).unwrap();

    for i in 0..=args.k_steps {
        let k = args.k_max * (i as f64) / (args.k_steps as f64);
        let mut model = sim::KuramotoModel::new(args.n_oscillators, k, args.dt);

        for _ in 0..args.steps {
            model.step();
        }

        let avg_window = 500;
        let mut r_sum = 0.0;
        for _ in 0..avg_window {
            model.step();
            r_sum += model.order_parameter();
        }
        let r = r_sum / avg_window as f64;

        println!("K = {:.4}, r = {:.4}", k, r);

        writer
            .write_record(&[format!("{:.6}", k), format!("{:.6}", r)])
            .unwrap();
        writer.flush().unwrap();
    }

    println!("\nResults written to {}", args.output.display());
}
