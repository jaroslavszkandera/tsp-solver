use crate::config::Config;
use crate::parser::TspInstance;
use rand::Rng;
use rand::prelude::IndexedRandom;

pub struct Ant {
    tour: Vec<usize>,
    visited: Vec<bool>,
    current_city_idx: usize,
    tour_length: f64,
}

impl Ant {
    pub fn new(start_node: usize, num_cities: usize) -> Self {
        let mut visited = vec![false; num_cities];
        visited[start_node] = true;
        Ant {
            tour: vec![start_node],
            visited,
            current_city_idx: start_node,
            tour_length: 0.0,
        }
    }

    pub fn visit_city(&mut self, city_idx: usize, distance: f64) {
        self.tour.push(city_idx);
        self.visited[city_idx] = true;
        self.current_city_idx = city_idx;
        self.tour_length += distance;
    }

    pub fn tour_completed(&self, num_cities: usize) -> bool {
        self.tour.len() == num_cities
    }
}

pub fn solve_tsp_aco(instance: &TspInstance, config: &Config) -> (Vec<usize>, f64) {
    let n_cities = instance.dimension;
    if n_cities == 0 {
        return (Vec::new(), 0.0);
    }
    if n_cities == 1 {
        return (vec![0], 0.0);
    }

    let dist_matrix = &instance.dist_matrix;

    let mut heuristic_matrix = vec![vec![0.0; n_cities]; n_cities];
    for i in 0..n_cities {
        for j in 0..n_cities {
            if i != j && dist_matrix[i][j] > 1e-9 {
                heuristic_matrix[i][j] = 1.0 / dist_matrix[i][j];
            } else {
                heuristic_matrix[i][j] = 1e-9; // avoid zero division
            }
        }
    }

    let mut pheromone_matrix = vec![vec![config.init_pheromone; n_cities]; n_cities];
    let mut best_tour_overall: Vec<usize> = Vec::new();
    let mut best_tour_length_overall = f64::MAX;

    let mut rng = rand::rng(); // Use thread_rng() for a more robust random number generator

    for iteration in 0..config.num_iters {
        let mut ants: Vec<Ant> = (0..config.num_ants.min(n_cities)) // Ensure num_ants <= n_cities
            .map(|_| Ant::new(rng.random_range(0..n_cities), n_cities)) // Use gen_range
            .collect();

        for ant_idx in 0..ants.len() {
            for _step in 1..n_cities {
                let current_city = ants[ant_idx].current_city_idx;
                let mut choices: Vec<(usize, f64)> = Vec::new();
                let mut current_choices_sum = 0.0;

                for next_city_idx in 0..n_cities {
                    if !ants[ant_idx].visited[next_city_idx] {
                        let pheromone = pheromone_matrix[current_city][next_city_idx];
                        let heuristic = heuristic_matrix[current_city][next_city_idx];
                        let prob_num = pheromone.powf(config.alpha) * heuristic.powf(config.beta);

                        if prob_num.is_finite() && prob_num > 1e-9 {
                            choices.push((next_city_idx, prob_num));
                            current_choices_sum += prob_num;
                        }
                    }
                }

                if choices.is_empty() || current_choices_sum < 1e-9 {
                    // Ant is stuck or no valid moves with positive probability
                    // Fallback: pick a random unvisited city
                    let unvisited: Vec<usize> = (0..n_cities)
                        .filter(|&i| !ants[ant_idx].visited[i])
                        .collect();
                    if let Some(&fallback_city) = unvisited.choose(&mut rng) {
                        ants[ant_idx]
                            .visit_city(fallback_city, dist_matrix[current_city][fallback_city]);
                    } else {
                        break; // Tour is effectively complete or truly stuck
                    }
                } else {
                    let rand_val = rng.random::<f64>() * current_choices_sum; // Use gen()
                    let mut cumulative_prob = 0.0;
                    let mut chosen_city = choices.last().unwrap().0;

                    for (city_idx, prob_val) in &choices {
                        cumulative_prob += *prob_val;
                        if rand_val <= cumulative_prob {
                            chosen_city = *city_idx;
                            break;
                        }
                    }
                    ants[ant_idx].visit_city(chosen_city, dist_matrix[current_city][chosen_city]);
                }
            }
            if ants[ant_idx].tour_completed(n_cities) {
                let last_city = ants[ant_idx].current_city_idx;
                let start_city = ants[ant_idx].tour[0];
                ants[ant_idx].tour_length += dist_matrix[last_city][start_city];
            }
        }

        // Pheromone evaporation
        for i in 0..n_cities {
            for j in 0..n_cities {
                pheromone_matrix[i][j] *= 1.0 - config.evap_rate;
                if pheromone_matrix[i][j] < 1e-5 {
                    pheromone_matrix[i][j] = 1e-5;
                }
            }
        }

        // Pheromone deposit
        for ant in &ants {
            if ant.tour_completed(n_cities) && ant.tour_length > 1e-9 {
                let pheromone_to_deposit = config.q_val / ant.tour_length;
                for k in 0..n_cities {
                    let city1_idx = ant.tour[k];
                    let city2_idx = ant.tour[(k + 1) % n_cities];
                    pheromone_matrix[city1_idx][city2_idx] += pheromone_to_deposit;
                    pheromone_matrix[city2_idx][city1_idx] += pheromone_to_deposit; // Symmetric
                }
            }
        }

        // Update best tour found so far in this iteration and overall
        let mut current_iter_best_tour_length = f64::MAX;

        for ant in &ants {
            if ant.tour_completed(n_cities) && ant.tour_length < best_tour_length_overall {
                best_tour_length_overall = ant.tour_length;
                best_tour_overall = ant.tour.clone();
            }
            if ant.tour_completed(n_cities) && ant.tour_length < current_iter_best_tour_length {
                current_iter_best_tour_length = ant.tour_length;
            }
        }

        // Elitist Ant System: Add extra pheromone for the global best tour
        if config.elitist_weight > 0.0
            && !best_tour_overall.is_empty()
            && best_tour_length_overall != f64::MAX
        {
            let elite_pheromone_deposit =
                config.elitist_weight * config.q_val / best_tour_length_overall;
            for k in 0..n_cities {
                let city1_idx = best_tour_overall[k];
                let city2_idx = best_tour_overall[(k + 1) % n_cities];
                pheromone_matrix[city1_idx][city2_idx] += elite_pheromone_deposit;
                pheromone_matrix[city2_idx][city1_idx] += elite_pheromone_deposit;
            }
        }

        if iteration % 100 == 0 || iteration == config.num_iters - 1 {
            println!(
                "Iter {}: Best tour length so far: {:.2}",
                iteration, best_tour_length_overall
            );
        }
    }
    (best_tour_overall, best_tour_length_overall.round())
}
