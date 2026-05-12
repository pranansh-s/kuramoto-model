use rand::Rng;
use std::f64::consts::PI;

/// dθᵢ/dt = ωᵢ + (K/N) Σⱼ sin(θⱼ − θᵢ)
/// r·e^(iψ) = (1/N) Σⱼ e^(iθⱼ)
pub struct KuramotoModel {
    omega: Vec<f64>,
    theta: Vec<f64>,
    k: f64,
    dt: f64,
    n: usize,
}

impl KuramotoModel {
    pub fn new(n: usize, k: f64, dt: f64) -> Self {
        let mut rng = rand::rng();
        let omega: Vec<f64> = (0..n).map(|_| rng.random_range(-1.0..1.0)).collect();
        let theta: Vec<f64> = (0..n).map(|_| rng.random_range(0.0..2.0 * PI)).collect();
        Self {
            omega,
            theta,
            k,
            dt,
            n,
        }
    }

    /// θᵢ += dθᵢ · dt
    pub fn step(&mut self) {
        let n = self.n as f64;
        let mut dtheta = vec![0.0; self.n];

        for i in 0..self.n {
            let mut coupling_sum = 0.0;
            for j in 0..self.n {
                coupling_sum += (self.theta[j] - self.theta[i]).sin();
            }
            dtheta[i] = self.omega[i] + (self.k / n) * coupling_sum;
        }

        for i in 0..self.n {
            self.theta[i] += dtheta[i] * self.dt;
            self.theta[i] = self.theta[i].rem_euclid(2.0 * PI);
        }
    }

    /// r = (1/N) |Σⱼ e^(iθⱼ)|
    pub fn order_parameter(&self) -> f64 {
        let (sum_cos, sum_sin) = self.theta.iter().fold((0.0, 0.0), |(sc, ss), &theta| {
            (sc + theta.cos(), ss + theta.sin())
        });
        let n = self.n as f64;
        (sum_cos.powi(2) + sum_sin.powi(2)).sqrt() / n
    }

    /// (r, ψ) — magnitude and phase of the mean field
    pub fn mean_field(&self) -> (f64, f64) {
        let (sc, ss) = self.theta.iter().fold((0.0, 0.0), |(sc, ss), &t| {
            (sc + t.cos(), ss + t.sin())
        });
        let n = self.n as f64;
        let r = (sc * sc + ss * ss).sqrt() / n;
        let psi = ss.atan2(sc);
        (r, psi)
    }

    pub fn phases(&self) -> &[f64] {
        &self.theta
    }

    #[allow(dead_code)]
    pub fn frequencies(&self) -> &[f64] {
        &self.omega
    }

    pub fn set_coupling(&mut self, k: f64) {
        self.k = k;
    }

    #[allow(dead_code)]
    pub fn n(&self) -> usize {
        self.n
    }
}
