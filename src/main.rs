mod sim;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "Kuramoto model simulation", long_about = None)]
struct Args {
    /// Number of oscillators
    #[arg(short, long, default_value_t = 200)]
    n_oscillators: usize,

    /// Number of time steps per K value
    #[arg(short, long, default_value_t = 5000)]
    steps: usize,

    /// Number of K values to sweep
    #[arg(long, default_value_t = 100)]
    k_steps: usize,

    /// Maximum coupling strength K
    #[arg(long, default_value_t = 5.0)]
    k_max: f64,

    /// Time step dt
    #[arg(long, default_value_t = 0.01)]
    dt: f64,

    /// Output CSV path
    #[arg(short, long, default_value = "data.csv")]
    output: PathBuf,
}

fn main() {
    let args = Args::parse();

    let mut writer = csv::Writer::from_path(&args.output).expect("Cannot create output file");
    writer.write_record(&["K", "r"]).unwrap();

    for i in 0..=args.k_steps {
        let k = args.k_max * (i as f64) / (args.k_steps as f64);

        let mut model = sim::KuramotoModel::new(args.n_oscillators, k, args.dt);

        // Run to steady state
        for _ in 0..args.steps {
            model.step();
        }

        // Average the order parameter over a window at the end to reduce noise
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
