pub mod attractor;
pub mod boids;
pub mod life;
pub mod logistic;
pub mod mandelbrot;
pub mod pendulum;
pub mod reaction;

use crate::scene::Scene;

pub fn all() -> Vec<Box<dyn Scene>> {
    vec![
        Box::new(attractor::Attractors::new()),
        Box::new(mandelbrot::Mandelbrot::new()),
        Box::new(reaction::Reaction::new()),
        Box::new(life::Life::new()),
        Box::new(boids::Boids::new()),
        Box::new(pendulum::Pendulum::new()),
        Box::new(logistic::Logistic::new()),
    ]
}

pub fn names() -> Vec<&'static str> {
    let list = all();
    list.iter().map(|s| s.name()).collect()
}

pub fn find(query: &str) -> Option<usize> {
    let q = query.to_lowercase();
    let list = all();
    list.iter().position(|s| s.name().starts_with(q.as_str()))
}
