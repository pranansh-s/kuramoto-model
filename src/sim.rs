use rand::Rng;
use std::f64::consts::PI;

#[derive(Clone, Debug)]
pub enum Topology {
    AllToAll,
    Ring(usize),
    SmallWorld(usize, f64),
    Random(f64),
}

impl Topology {
    pub fn label(&self) -> String {
        match self {
            Topology::AllToAll => "All-to-all".into(),
            Topology::Ring(k) => format!("Ring (k={})", k),
            Topology::SmallWorld(k, p) => format!("Small-world (k={}, p={:.2})", k, p),
            Topology::Random(p) => format!("Random (p={:.2})", p),
        }
    }
}

fn build_adjacency(n: usize, topology: &Topology) -> Vec<Vec<usize>> {
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut rng = rand::rng();

    match topology {
        Topology::AllToAll => {
            for i in 0..n {
                for j in 0..n {
                    if i != j {
                        adj[i].push(j);
                    }
                }
            }
        }
        Topology::Ring(k) => {
            for i in 0..n {
                for offset in 1..=*k {
                    let j = (i + offset) % n;
                    if !adj[i].contains(&j) {
                        adj[i].push(j);
                    }
                    let j = (i + n - offset) % n;
                    if !adj[i].contains(&j) {
                        adj[i].push(j);
                    }
                }
            }
        }
        Topology::SmallWorld(k, p) => {
            // Watts-Strogatz: start with ring, rewire with probability p
            for i in 0..n {
                for offset in 1..=*k {
                    adj[i].push((i + offset) % n);
                    adj[i].push((i + n - offset) % n);
                }
            }
            for i in 0..n {
                let neighbors: Vec<usize> = adj[i].clone();
                for &j in &neighbors {
                    if rng.random_range(0.0..1.0) < *p {
                        adj[i].retain(|&x| x != j);
                        adj[j].retain(|&x| x != i);
                        // pick a random new target (not self, not already connected)
                        let mut new_j = rng.random_range(0..n);
                        let mut tries = 0;
                        while (new_j == i || adj[i].contains(&new_j)) && tries < n {
                            new_j = rng.random_range(0..n);
                            tries += 1;
                        }
                        if new_j != i && !adj[i].contains(&new_j) {
                            adj[i].push(new_j);
                            adj[new_j].push(i);
                        }
                    }
                }
            }
        }
        Topology::Random(p) => {
            for i in 0..n {
                for j in (i + 1)..n {
                    if rng.random_range(0.0..1.0) < *p {
                        adj[i].push(j);
                        adj[j].push(i);
                    }
                }
            }
        }
    }
    adj
}

/// dθi/dt = ωi + (K/N) Σ_{j∈adj(i)} sin(θj − θi)
/// r·e^(iψ) = (1/N) Σj e^(iθj)
pub struct KuramotoModel {
    omega: Vec<f64>,
    theta: Vec<f64>,
    k: f64,
    dt: f64,
    n: usize,
    adj: Vec<Vec<usize>>,
    #[allow(dead_code)]
    topology: Topology,
}

impl KuramotoModel {
    pub fn new(n: usize, k: f64, dt: f64, topology: Topology) -> Self {
        let mut rng = rand::rng();
        let omega: Vec<f64> = (0..n).map(|_| rng.random_range(-1.0..1.0)).collect();
        let theta: Vec<f64> = (0..n).map(|_| rng.random_range(0.0..2.0 * PI)).collect();
        let adj = build_adjacency(n, &topology);
        Self {
            omega,
            theta,
            k,
            dt,
            n,
            adj,
            topology,
        }
    }

    /// θi += dθi · dt
    pub fn step(&mut self) {
        let n = self.n as f64;
        let mut dtheta = vec![0.0; self.n];

        for i in 0..self.n {
            let mut coupling_sum = 0.0;
            for &j in &self.adj[i] {
                coupling_sum += (self.theta[j] - self.theta[i]).sin();
            }
            dtheta[i] = self.omega[i] + (self.k / n) * coupling_sum;
        }

        for i in 0..self.n {
            self.theta[i] += dtheta[i] * self.dt;
            self.theta[i] = self.theta[i].rem_euclid(2.0 * PI);
        }
    }

    /// r = (1/N) |Σj e^(iθj)|
    pub fn order_parameter(&self) -> f64 {
        let (sum_cos, sum_sin) = self.theta.iter().fold((0.0, 0.0), |(sc, ss), &theta| {
            (sc + theta.cos(), ss + theta.sin())
        });
        let n = self.n as f64;
        (sum_cos.powi(2) + sum_sin.powi(2)).sqrt() / n
    }

    /// (r, ψ)
    pub fn mean_field(&self) -> (f64, f64) {
        let (sc, ss) = self
            .theta
            .iter()
            .fold((0.0, 0.0), |(sc, ss), &t| (sc + t.cos(), ss + t.sin()));
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
    pub fn topology(&self) -> &Topology {
        &self.topology
    }

    #[allow(dead_code)]
    pub fn n(&self) -> usize {
        self.n
    }
}
