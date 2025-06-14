pub mod config;
pub mod parser;
pub mod solver;
pub mod utils;

pub use config::Config;
pub use parser::{EdgeWeightFormat, EdgeWeightType, Node, TspInstance, parse_tsp_file};
pub use solver::{Ant, solve_tsp_aco};
pub use utils::{evaluate_solution, load_optimal_solutions};

use std::error::Error;

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    println!("\nRustACO - Ant Colony Optimization for TSP");
    println!("========================================");
    println!("\n ACO Configuration:");
    println!("  Iterations: {}", config.num_iters);
    println!("  Number of Ants: {}", config.num_ants);
    println!("  Alpha (pheromone influence): {:.2}", config.alpha);
    println!("  Beta (heuristic influence): {:.2}", config.beta);
    println!("  Evaporation Rate (rho): {:.2}", config.evap_rate);
    println!("  Q Value (pheromone deposit factor): {:.2}", config.q_val);
    println!("  Initial Pheromone: {:.2}", config.init_pheromone);
    println!("  Elitist Weight: {:.2}", config.elitist_weight);
    println!("  Min Pheromone Value: {:.0e}", config.min_pheromone_val);

    let file_path = config
        .file_path
        .as_deref()
        .ok_or("File path not provided in config")?;
    println!("\n Parsing TSP file: {}...", file_path);

    let instance = match parse_tsp_file(file_path) {
        Ok(inst) => {
            println!("  Successfully parsed: {}", inst.name);
            println!("  Problem Type: {}", inst.tsp_type);
            if !inst.comment.is_empty() {
                println!("  Comment: {}", inst.comment);
            }
            println!("  Dimension: {}", inst.dimension);
            println!("  Edge Weight Type: {:?}", inst.edge_weight_type);
            if let Some(format) = &inst.edge_weight_format {
                if !matches!(format, EdgeWeightFormat::Unknown(_)) {
                    println!("  Edge Weight Format: {:?}", format);
                }
            }
            if inst.dimension == 0 {
                return Err("Problem dimension is 0. Cannot solve.".into());
            }
            inst
        }
        Err(e) => {
            return Err(format!("Error parsing TSPLIB file: {}", e).into());
        }
    };

    println!("\n Starting ACO to solve TSP for {}...", instance.name);
    let start_time = std::time::Instant::now();
    let (best_tour_indices, best_tour_length) = solve_tsp_aco(&instance, config);
    let duration = start_time.elapsed();

    println!("\n --- ACO Results for {} ---", instance.name);
    println!("   Time taken: {:.2?}", duration);

    if best_tour_length == 0.0 && (best_tour_indices.is_empty() || instance.dimension > 1) {
        println!("   No tour found or tour length is zero for a multi-node problem.");
    } else {
        println!("   Best tour length found: {:.2}", best_tour_length);
    }

    if !best_tour_indices.is_empty() {
        let valid_indices = best_tour_indices
            .iter()
            .all(|&idx| idx < instance.dimension);

        if valid_indices && best_tour_indices.len() == instance.dimension {
            if best_tour_indices.len() <= 30 {
                if let Some(nodes) = &instance.node_coords {
                    let display_tour: Vec<usize> = best_tour_indices.iter().map(|&idx| {
                        nodes.get(idx).map_or_else(|| {
                            eprintln!("Warning: Solver index {} out of bounds for node_coords (len {})", idx, nodes.len());
                            idx + 1
                        }, |node| node.id)
                    }).collect();
                    println!("   Route (Node IDs): {:?}", display_tour);
                } else {
                    let display_tour_indices: Vec<usize> =
                        best_tour_indices.iter().map(|&idx| idx + 0).collect();
                    println!(
                        "   Route (0-based City Indices): {:?}",
                        display_tour_indices
                    );
                }
            } else {
                println!(
                    "  Tour is too long to print ({} cities).",
                    best_tour_indices.len()
                );
            }
        } else if !best_tour_indices.is_empty() {
            println!(
                "   Partial or invalid tour found: {:?} (Length: {})",
                best_tour_indices,
                best_tour_indices.len()
            );
        }
    } else if instance.dimension > 0 {
        println!("  No tour found by the solver.");
    }

    let solutions_file_path = "tsplib/solutions";
    match load_optimal_solutions(solutions_file_path) {
        Ok(optimal_solutions) => {
            let problem_base_name = instance.name.split('.').next().unwrap_or(&instance.name);
            let (optimal_len_opt, diff_opt) =
                evaluate_solution(problem_base_name, best_tour_length, &optimal_solutions);

            if let Some(optimal_len) = optimal_len_opt {
                println!(
                    "   Optimal solution for {}: {:.0}",
                    problem_base_name, optimal_len
                );
                if let Some(percentage_diff) = diff_opt {
                    if best_tour_length > 0.0 {
                        println!(
                            "   ACO solution is {:.2}% away from optimal.",
                            percentage_diff
                        );
                    } else {
                        println!(
                            "   Cannot calculate deviation from optimal as no valid tour was found by ACO."
                        );
                    }
                }
            } else {
                println!(
                    "  ℹ️ No optimal solution found in '{}' for '{}'",
                    solutions_file_path, problem_base_name
                );
            }
        }
        Err(e) => {
            eprintln!("   Could not load optimal solutions: {}", e);
        }
    }
    println!("========================================");
    Ok(())
}
