pub mod config;
pub mod parser;
pub mod solver;
pub mod utils;

pub use config::Config;
pub use parser::{EdgeWeightType, Node, TspInstance, parse_tsp_file};
pub use solver::{Ant, solve_tsp_aco};
pub use utils::{evaluate_solution, load_optimal_solutions};

use std::error::Error;

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    println!("\nACO Configuration:");
    println!(" Iterations: {}", config.num_iters);
    println!(" Number of Ants: {}", config.num_ants);
    println!(" Alpha (pheromone influence): {:.2}", config.alpha);
    println!(" Beta (heuristic influence): {:.2}", config.beta);
    println!(" Evaporation Rate (rho): {:.2}", config.evap_rate);
    println!(" Q Value (pheromone deposit): {:.2}", config.q_val);
    println!(" Initial Pheromone: {:.2}", config.init_pheromone);
    println!(" Elitist Weight: {:.2}", config.elitist_weight);

    println!(
        "\nStarting ACO to solve TSP for {}...",
        config
            .file_path
            .as_deref()
            .ok_or("File path not provided in config")?
    );
    let instance = match parse_tsp_file(
        config
            .file_path
            .as_deref()
            .ok_or("File path not provided in config")?,
    ) {
        Ok(inst) => {
            println!("Successfully parsed: {}", inst.name);
            println!(" Dimension: {}", inst.dimension);
            println!(" Edge Weight Type: {:?}", inst.edge_weight_type);
            println!(" TSP Type: {}", inst.tsp_type);
            println!(" Comment {}", inst.comment);
            if inst.dimension == 0 {
                return Err("Problem dimension is 0. Cannot solve.".into());
            }
            inst
        }
        Err(e) => {
            return Err(format!("Error parsing TSPLIB file: {}", e).into());
        }
    };

    println!("\nStarting ACO to solve TSP for {}...", instance.name);
    let start_time = std::time::Instant::now(); // Use std::time::Instant
    let (best_tour_indices, best_tour_length) = solve_tsp_aco(&instance, config);
    let duration = start_time.elapsed();

    println!("\n--- ACO Results for {} ---", instance.name);
    println!(" Time taken: {:.2?}", duration);
    println!(" Best tour length found: {:.2}", best_tour_length);

    if !best_tour_indices.is_empty() {
        if best_tour_indices.len() <= 30 {
            if let Some(nodes) = &instance.node_coords {
                let display_tour: Vec<usize> =
                    best_tour_indices.iter().map(|&idx| nodes[idx].id).collect();
                println!("Route (display_tour): {:?}", display_tour);
            } else {
                println!("Route (best_tour_indices): {:?}", best_tour_indices);
            }
        } else {
            println!(
                " Tour is too long to print ({} cities).",
                best_tour_indices.len()
            );
        }
    } else {
        println!(" No tour found.");
    }

    let solutions_file_path = "tsplib/solutions";
    match load_optimal_solutions(solutions_file_path) {
        Ok(optimal_solutions) => {
            let problem_base_name = instance.name.split('.').next().unwrap_or(&instance.name);
            let (optimal_len_opt, diff_opt) =
                evaluate_solution(problem_base_name, best_tour_length, &optimal_solutions);

            if let Some(optimal_len) = optimal_len_opt {
                println!(
                    " Optimal solution for {}: {:.0}",
                    problem_base_name, optimal_len
                );
                if let Some(percentage_diff) = diff_opt {
                    println!(
                        " ACO solution is {:.2}% away from optimal.",
                        percentage_diff
                    );
                }
            } else {
                println!(
                    " No optimal solution found in {} for '{}'",
                    solutions_file_path, problem_base_name
                );
            }
        }
        Err(e) => {
            eprintln!(" Could not load optimal solutions: {}", e);
        }
    }

    Ok(())
}
