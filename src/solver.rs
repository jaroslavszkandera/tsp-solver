use crate::config::Config;
use crate::parser::TspInstance;
use rand::Rng;
use rand::prelude::IndexedRandom;
use rayon::prelude::*;

pub struct Ant {
    tour: Vec<usize>,
    visited: Vec<bool>,
    current_node_idx: usize,
    tour_length: f64,
}

impl Ant {
    pub fn new(start_node: usize, num_nodes: usize) -> Self {
        let mut visited = vec![false; num_nodes];
        if num_nodes > 0 {
            visited[start_node] = true;
        }
        let mut tour = Vec::with_capacity(num_nodes);
        if num_nodes > 0 {
            tour.push(start_node);
        }
        Ant {
            tour,
            visited,
            current_node_idx: start_node,
            tour_length: 0.0,
        }
    }

    pub fn visit_node(&mut self, node_idx: usize, distance: f64) {
        self.tour.push(node_idx);
        self.visited[node_idx] = true;
        self.current_node_idx = node_idx;
        self.tour_length += distance;
    }

    #[inline]
    pub fn tour_completed(&self, num_nodes: usize) -> bool {
        self.tour.len() == num_nodes
    }
}

pub fn solve_tsp_aco(instance: &TspInstance, config: &Config) -> (Vec<usize>, f64) {
    let n_nodes = instance.dimension;
    if n_nodes == 0 {
        return (Vec::new(), 0.0);
    }
    if n_nodes == 1 {
        return (vec![0], 0.0);
    }

    let dist_matrix = &instance.dist_matrix;
    let heuristic_matrix = {
        let mut matrix = vec![vec![0.0f64; n_nodes]; n_nodes];
        for i in 0..n_nodes {
            for j in 0..n_nodes {
                if i != j {
                    let dist = dist_matrix[i][j];
                    matrix[i][j] = if dist > 1e-9 { 1.0 / dist } else { 1.0 / 1e-9 };
                }
            }
        }
        matrix
    };

    let mut pheromone_matrix = vec![vec![config.init_pheromone; n_nodes]; n_nodes];
    let mut best_tour_overall: Vec<usize> = Vec::with_capacity(n_nodes);
    let mut best_tour_length_overall = f64::MAX;

    for iteration in 0..config.num_iters {
        let ants: Vec<Ant> = (0..config.num_ants.min(n_nodes))
            .into_par_iter()
            .map(|_| {
                let mut rng = rand::rng();
                let start_node = if n_nodes > 0 {
                    rng.random_range(0..n_nodes)
                } else {
                    0
                };
                let mut ant = Ant::new(start_node, n_nodes);

                for _step in 1..n_nodes {
                    let current_node = ant.current_node_idx;
                    let mut choices: Vec<(usize, f64)> = Vec::with_capacity(n_nodes);
                    let mut current_choices_sum = 0.0;

                    for next_node_idx in 0..n_nodes {
                        if !ant.visited[next_node_idx] {
                            // Read from shared matrices
                            let pheromone = pheromone_matrix[current_node][next_node_idx];
                            let heuristic = heuristic_matrix[current_node][next_node_idx];
                            let prob_num =
                                pheromone.powf(config.alpha) * heuristic.powf(config.beta);

                            if prob_num.is_finite() && prob_num > 1e-12 {
                                choices.push((next_node_idx, prob_num));
                                current_choices_sum += prob_num;
                            }
                        }
                    }

                    if choices.is_empty() || current_choices_sum < 1e-12 {
                        let unvisited: Vec<usize> =
                            (0..n_nodes).filter(|&i| !ant.visited[i]).collect();
                        if let Some(&fallback_node) = unvisited.choose(&mut rng) {
                            ant.visit_node(fallback_node, dist_matrix[current_node][fallback_node]);
                        } else {
                            break;
                        }
                    } else {
                        let rand_val = rng.random::<f64>() * current_choices_sum;
                        let mut cumulative_prob = 0.0;
                        let mut chosen_node = choices[0].0;
                        for (node_idx, prob_val) in &choices {
                            cumulative_prob += *prob_val;
                            if rand_val <= cumulative_prob {
                                chosen_node = *node_idx;
                                break;
                            }
                        }
                        ant.visit_node(chosen_node, dist_matrix[current_node][chosen_node]);
                    }
                }
                // Complete the tour by adding distance to return to start
                if ant.tour_completed(n_nodes) {
                    let last_node = ant.current_node_idx;
                    let start_node = ant.tour[0];
                    ant.tour_length += dist_matrix[last_node][start_node];
                }
                ant // Return the fully constructed ant
            })
            .collect(); // Collect all ants processed

        // --- Pheromone Evaporation ---
        pheromone_matrix.par_iter_mut().for_each(|row| {
            for val in row.iter_mut() {
                *val *= 1.0 - config.evap_rate;
                if *val < config.min_pheromone_val {
                    *val = config.min_pheromone_val;
                }
            }
        });

        // --- Sequential Pheromone Deposit & Best Tour Update ---
        for ant in &ants {
            // Pheromone Deposit
            if ant.tour_completed(n_nodes) && ant.tour_length > 1e-9 {
                let pheromone_to_deposit = config.q_val / ant.tour_length;
                for k in 0..n_nodes {
                    let node1_idx = ant.tour[k];
                    let node2_idx = ant.tour[(k + 1) % n_nodes];
                    if node1_idx < n_nodes && node2_idx < n_nodes {
                        pheromone_matrix[node1_idx][node2_idx] += pheromone_to_deposit;
                        pheromone_matrix[node2_idx][node1_idx] += pheromone_to_deposit;
                    }
                }
            }

            // Update Best Tour
            if ant.tour_completed(n_nodes) && ant.tour_length < best_tour_length_overall {
                best_tour_length_overall = ant.tour_length;
                best_tour_overall.clone_from(&ant.tour);
            }
        }

        // --- Elitist Ant System Update ---
        if config.elitist_weight > 0.0
            && !best_tour_overall.is_empty()
            && best_tour_length_overall < f64::MAX - 1e-9
        {
            let elite_pheromone_amount =
                config.elitist_weight * config.q_val / best_tour_length_overall;
            for k in 0..n_nodes {
                let node1_idx = best_tour_overall[k];
                let node2_idx = best_tour_overall[(k + 1) % n_nodes];
                if node1_idx < n_nodes && node2_idx < n_nodes {
                    pheromone_matrix[node1_idx][node2_idx] += elite_pheromone_amount;
                    pheromone_matrix[node2_idx][node1_idx] += elite_pheromone_amount;
                }
            }
        }

        if iteration % 100 == 0 || iteration == config.num_iters - 1 {
            if best_tour_length_overall == f64::MAX {
                println!("Iter {}: No complete tour found yet.", iteration);
            } else {
                println!(
                    "Iter {}: Best tour length so far: {:.2}",
                    iteration, best_tour_length_overall
                );
            }
        }
    }

    let final_length = if best_tour_length_overall == f64::MAX {
        0.0
    } else {
        best_tour_length_overall.round()
    };
    (best_tour_overall, final_length)
}
