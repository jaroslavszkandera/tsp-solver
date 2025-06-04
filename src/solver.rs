use crate::config::Config;
use crate::parser::TspInstance;
use rand::Rng;
use rand::prelude::IndexedRandom;

pub struct Ant {
    tour: Vec<usize>,
    visited: Vec<bool>,
    current_node_idx: usize,
    tour_length: f64,
}

impl Ant {
    pub fn new(start_node: usize, num_nodes: usize) -> Self {
        let mut visited = vec![false; num_nodes];
        visited[start_node] = true;
        Ant {
            tour: vec![start_node],
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
        return (
            vec![
                instance
                    .node_coords
                    .as_ref()
                    .and_then(|nc| nc.get(0).map(|n| n.id - 1))
                    .unwrap_or(0),
            ],
            0.0,
        );
    }

    let dist_matrix = &instance.dist_matrix;

    let mut heuristic_matrix = vec![vec![0.0; n_nodes]; n_nodes];
    for i in 0..n_nodes {
        for j in 0..n_nodes {
            if i != j && dist_matrix[i][j] > 1e-9 {
                heuristic_matrix[i][j] = 1.0 / dist_matrix[i][j];
            } else {
                heuristic_matrix[i][j] = 1e-9; // Avoid division by zero
            }
        }
    }

    let mut pheromone_matrix = vec![vec![config.init_pheromone; n_nodes]; n_nodes];
    let mut best_tour_overall: Vec<usize> = Vec::new();
    let mut best_tour_length_overall = f64::MAX;

    let mut rng = rand::rng();

    for iteration in 0..config.num_iters {
        let mut ants: Vec<Ant> = (0..config.num_ants.min(n_nodes))
            .map(|_| Ant::new(rng.random_range(0..n_nodes), n_nodes))
            .collect();

        for ant_idx in 0..ants.len() {
            for _step in 1..n_nodes {
                let current_node = ants[ant_idx].current_node_idx;
                let mut choices: Vec<(usize, f64)> = Vec::new();
                let mut current_choices_sum = 0.0;

                for next_node_idx in 0..n_nodes {
                    if !ants[ant_idx].visited[next_node_idx] {
                        let pheromone = pheromone_matrix[current_node][next_node_idx];
                        let heuristic = heuristic_matrix[current_node][next_node_idx];
                        let prob_num = pheromone.powf(config.alpha) * heuristic.powf(config.beta);

                        if prob_num.is_finite() && prob_num > 1e-12 {
                            choices.push((next_node_idx, prob_num));
                            current_choices_sum += prob_num;
                        }
                    }
                }

                if choices.is_empty() || current_choices_sum < 1e-12 {
                    // If no valid choices
                    let unvisited: Vec<usize> = (0..n_nodes)
                        .filter(|&i| !ants[ant_idx].visited[i])
                        .collect();
                    if let Some(&fallback_node) = unvisited.choose(&mut rng) {
                        ants[ant_idx]
                            .visit_node(fallback_node, dist_matrix[current_node][fallback_node]);
                    } else {
                        break; // Ant is truly stuck or tour is prematurely complete.
                    }
                } else {
                    let rand_val = rng.random::<f64>() * current_choices_sum; // Use gen()
                    let mut cumulative_prob = 0.0;
                    let mut chosen_node = choices.last().map_or(
                        (0..n_nodes)
                            .find(|&i| !ants[ant_idx].visited[i])
                            .unwrap_or(ants[ant_idx].current_node_idx),
                        |choice| choice.0,
                    );

                    for (node_idx, prob_val) in &choices {
                        cumulative_prob += *prob_val;
                        if rand_val <= cumulative_prob {
                            chosen_node = *node_idx;
                            break;
                        }
                    }
                    ants[ant_idx].visit_node(chosen_node, dist_matrix[current_node][chosen_node]);
                }
            }
            if ants[ant_idx].tour_completed(n_nodes) {
                let last_node = ants[ant_idx].current_node_idx;
                let start_node = ants[ant_idx].tour[0];
                ants[ant_idx].tour_length += dist_matrix[last_node][start_node];
            }
        }

        // Pheromone evaporation
        for i in 0..n_nodes {
            for j in 0..n_nodes {
                pheromone_matrix[i][j] *= 1.0 - config.evap_rate;
                if pheromone_matrix[i][j] < config.min_pheromone_val {
                    pheromone_matrix[i][j] = config.min_pheromone_val;
                }
            }
        }

        // Pheromone deposit
        for ant in &ants {
            if ant.tour_completed(n_nodes) && ant.tour_length > 1e-9 {
                let pheromone_to_deposit = config.q_val / ant.tour_length;
                for k in 0..n_nodes {
                    let node1_idx = ant.tour[k];
                    let node2_idx = ant.tour[(k + 1) % n_nodes]; // Wrap around for the last edge

                    if node1_idx < n_nodes && node2_idx < n_nodes {
                        pheromone_matrix[node1_idx][node2_idx] += pheromone_to_deposit;
                        pheromone_matrix[node2_idx][node1_idx] += pheromone_to_deposit;
                    }
                }
            }
        }

        // Update best tour found so far
        for ant in &ants {
            if ant.tour_completed(n_nodes) && ant.tour_length < best_tour_length_overall {
                best_tour_length_overall = ant.tour_length;
                best_tour_overall = ant.tour.clone();
            }
        }

        // Elitist Ant System: Add extra pheromone for the global best tour
        if config.elitist_weight > 0.0
            && !best_tour_overall.is_empty()
            && best_tour_length_overall > 1e-9
            && best_tour_length_overall != f64::MAX
        {
            let elite_pheromone_deposit =
                config.elitist_weight * config.q_val / best_tour_length_overall;
            for k in 0..n_nodes {
                let node1_idx = best_tour_overall[k];
                let node2_idx = best_tour_overall[(k + 1) % n_nodes];
                if node1_idx < n_nodes && node2_idx < n_nodes {
                    pheromone_matrix[node1_idx][node2_idx] += elite_pheromone_deposit;
                    pheromone_matrix[node2_idx][node1_idx] += elite_pheromone_deposit;
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
