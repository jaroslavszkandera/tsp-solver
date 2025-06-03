use rand::Rng;
use rand::prelude::IndexedRandom;
use std::collections::HashMap;
use std::fs::File as StdFile;
use std::io::{BufRead, BufReader as StdBufReader};

fn calc_euc_2d_dist(n1: &Node, n2: &Node) -> f64 {
    let dx = n1.x - n2.x;
    let dy = n1.y - n2.y;
    ((dx * dx + dy * dy).sqrt() + 0.5).floor()
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeWeightType {
    Euc2D,
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize,
    pub x: f64,
    pub y: f64,
}

pub struct TspInstance {
    pub name: String,
    pub tsp_type: String,
    pub comment: String,
    pub dimension: usize,
    pub edge_weight_type: EdgeWeightType,
    pub node_coords: Option<Vec<Node>>,
    // pub edge_weights_format: Option<String>,
    // pub explicit_edge_weights: Option<Vec<f64>>,
    pub dist_matrix: Vec<Vec<f64>>,
}

impl TspInstance {
    #[allow(dead_code)]
    pub fn get_dist(&self, city1_idx: usize, city2_idx: usize) -> f64 {
        if city1_idx >= self.dimension || city2_idx >= self.dimension {
            panic!("City index out of bounds");
        }
        self.dist_matrix[city1_idx][city2_idx]
    }
}

pub fn parse_tsp_file(file_path: &str) -> Result<TspInstance, String> {
    let file = StdFile::open(file_path)
        .map_err(|e| format!("Failed to open file {}: {}", file_path, e))?;
    let reader = StdBufReader::new(file);

    let mut name = String::new();
    let mut tsp_type = String::new();
    let mut comment = String::new();
    let mut dimension = 0;
    let mut edge_weight_type_str = String::new();
    let mut node_coords_vec: Vec<Node> = Vec::new();

    let mut reading_node_coords = false;
    let mut current_line_num = 0;

    for line_result in reader.lines() {
        current_line_num += 1;
        let line = line_result
            .map_err(|e| format!("Error reading line {}: {}", current_line_num, e))?
            .trim()
            .to_string();

        if line == "EOF" || line.is_empty() {
            if reading_node_coords && node_coords_vec.len() == dimension {
                reading_node_coords = false;
            } else if line == "EOF" {
                break;
            }
            continue;
        }

        if reading_node_coords {
            if line == "DISPLAY_DATA_SECTION"
                || line == "TOUR_SECTION"
                || line == "EDGE_WEIGHT_SECTION"
                || line.starts_with("EDGE_DATA_SECTION")
            {
                reading_node_coords = false;
                // TODO: re-evaluate 'line'
            } else {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let id = parts[0].parse::<usize>().map_err(|e| {
                        format!(
                            "L{}: Invalid node id: {} on line '{}'",
                            current_line_num, e, line
                        )
                    })?;
                    let x = parts[1].parse::<f64>().map_err(|e| {
                        format!(
                            "L{}: Invalid x coord: {} on line '{}'",
                            current_line_num, e, line
                        )
                    })?;
                    let y = parts[2].parse::<f64>().map_err(|e| {
                        format!(
                            "L{}: Invalid y coord: {} on line '{}'",
                            current_line_num, e, line
                        )
                    })?;
                    node_coords_vec.push(Node { id, x, y });
                    if node_coords_vec.len() == dimension {
                        reading_node_coords = false;
                    }
                } else {
                    return Err(format!(
                        "L{}: Malformed node coord line: {}",
                        current_line_num, line
                    ));
                }
                continue;
            }
        }

        let parts: Vec<&str> = line.splitn(2, ':').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let key = parts[0];
            let value = parts[1];
            match key {
                "NAME" => name = value.to_string(),
                "TYPE" => tsp_type = value.to_string(),
                "COMMENT" => comment = value.to_string(),
                "DIMENSION" => {
                    dimension = value
                        .parse::<usize>()
                        .map_err(|e| format!("L{}: Invalid dimension: {}", current_line_num, e))?
                }
                "EDGE_WEIGHT_TYPE" => edge_weight_type_str = value.to_string(),
                "NODE_COORD_SECTION" => reading_node_coords = true,
                _ => {} // Ignore other keywords
            }
        } else if line == "NODE_COORD_SECTION" {
            reading_node_coords = true;
        }
    }

    if dimension == 0 {
        return Err("DIMENSION not found or is zero.".to_string());
    }

    let ewt = match edge_weight_type_str.as_str() {
        "EUC_2D" => EdgeWeightType::Euc2D,
        s => EdgeWeightType::Unknown(s.to_string()),
    };

    if (ewt == EdgeWeightType::Euc2D) && node_coords_vec.len() != dimension {
        return Err(format!(
            "Mismatch: DIMENSION ({}) vs found node coordinates ({}). Type: {:?}",
            dimension,
            node_coords_vec.len(),
            ewt
        ));
    }

    let mut dist_matrix = vec![vec![0.0; dimension]; dimension];

    match ewt {
        EdgeWeightType::Euc2D => {
            for i in 0..dimension {
                for j in 0..dimension {
                    if i == j {
                        continue;
                    }
                    dist_matrix[i][j] = calc_euc_2d_dist(&node_coords_vec[i], &node_coords_vec[j]);
                }
            }
        }
        EdgeWeightType::Unknown(ref s) => return Err(format!("Unknown edge weight type: {}", s)),
    }

    Ok(TspInstance {
        name,
        tsp_type,
        comment,
        dimension,
        edge_weight_type: ewt,
        node_coords: Some(node_coords_vec),
        dist_matrix,
    })
}

#[derive(Debug, Clone)]
pub struct ACOConfig {
    pub num_ants: usize,
    pub num_iterations: usize,
    pub alpha: f64,             // Pheromone influence
    pub beta: f64,              // Heuristic influence
    pub evap_rate: f64,         // Rho
    pub q_val: f64,             // Pheromone deposit factor
    pub initial_pheromone: f64, // Initial pheromone
    pub elitist_weight: f64,    // Weight for elitist ant pheromone deposit (0 for no elitism)
}

impl Default for ACOConfig {
    fn default() -> Self {
        ACOConfig {
            num_ants: 10, // N, N*2, N/2
            num_iterations: 1000,
            alpha: 0.5,             // 0.5 - 2.0
            beta: 5.0,              // 2.0 - 5.0
            evap_rate: 0.5,         // 0.1 - 0.5
            q_val: 100.0,           // Pheromone Deposit Factor
            initial_pheromone: 0.2, // != 0; 0.1 - 0.2
            // e.g., 1.0 means global best tour gets `elitist_weight * Q / L_gb`
            elitist_weight: 1.0,
        }
    }
}

struct Ant {
    tour: Vec<usize>,
    visited: Vec<bool>,
    current_city_idx: usize,
    tour_length: f64,
}

impl Ant {
    fn new(start_node: usize, num_cities: usize) -> Self {
        let mut visited = vec![false; num_cities];
        visited[start_node] = true;
        Ant {
            tour: vec![start_node],
            visited,
            current_city_idx: start_node,
            tour_length: 0.0,
        }
    }

    fn visit_city(&mut self, city_idx: usize, distance: f64) {
        self.tour.push(city_idx);
        self.visited[city_idx] = true;
        self.current_city_idx = city_idx;
        self.tour_length += distance;
    }

    fn tour_completed(&self, num_cities: usize) -> bool {
        self.tour.len() == num_cities
    }
}

pub fn solve_tsp_aco(instance: &TspInstance, config: &ACOConfig) -> (Vec<usize>, f64) {
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

    let mut pheromone_matrix = vec![vec![config.initial_pheromone; n_cities]; n_cities];
    let mut best_tour_overall: Vec<usize> = Vec::new();
    let mut best_tour_length_overall = f64::MAX;

    let mut rng = rand::rng();

    for iteration in 0..config.num_iterations {
        let mut ants: Vec<Ant> =
            (0..config.num_ants.min(n_cities)) // Ensure num_ants <= n_cities
                .map(|_| Ant::new(rng.random_range(0..n_cities), n_cities))
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
                            // Check for valid positive probability
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
                    let rand_val = rng.random::<f64>() * current_choices_sum;
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
                    // Prevent pheromones from becoming too small (min pheromone implicitly)
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
                    // Handles wrap-around for the last edge
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

        if iteration % 100 == 0 || iteration == config.num_iterations - 1 {
            println!(
                "Iter {}: Best tour length so far: {:.2}",
                iteration, best_tour_length_overall
            );
        }
    }
    (best_tour_overall, best_tour_length_overall.round())
}

pub fn load_optimal_solutions(file_path: &str) -> Result<HashMap<String, f64>, String> {
    let file = StdFile::open(file_path)
        .map_err(|e| format!("Failed to open solutions file {}: {}", file_path, e))?;
    let reader = StdBufReader::new(file);
    let mut solutions = HashMap::new();

    for line_result in reader.lines() {
        let line = line_result.map_err(|e| format!("Error reading solution line: {}", e))?;
        let parts: Vec<&str> = line.split(':').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let name_part = parts[0];
            let clean_name = name_part
                .split_whitespace()
                .next()
                .unwrap_or(name_part)
                .to_lowercase();

            let value_str_full = parts[1];
            let value_str_numeric = value_str_full
                .split_whitespace()
                .next()
                .unwrap_or(value_str_full);

            let value = value_str_numeric.parse::<f64>().map_err(|e| {
                format!(
                    "Invalid solution value for {} (from '{}'): {}",
                    clean_name, value_str_full, e
                )
            })?;
            solutions.insert(clean_name, value);
        }
    }
    Ok(solutions)
}

pub fn evaluate_solution(
    problem_name: &str,
    found_length: f64,
    optimal_solutions: &HashMap<String, f64>,
) -> (Option<f64>, Option<f64>) {
    let key_name = problem_name.to_lowercase();
    if let Some(optimal_length) = optimal_solutions.get(&key_name) {
        let percentage_diff = if *optimal_length == 0.0 {
            if found_length == 0.0 {
                0.0
            } else {
                f64::INFINITY
            }
        } else {
            ((found_length - optimal_length) / optimal_length) * 100.0
        };
        (Some(*optimal_length), Some(percentage_diff))
    } else {
        (None, None)
    }
}
