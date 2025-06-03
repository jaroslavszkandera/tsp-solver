use std::time::Instant;

use tsp_solver::{
    ACOConfig, evaluate_solution, load_optimal_solutions, parse_tsp_file, solve_tsp_aco,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "Usage: {} <tsplib_file_path> [iterations] [ants] [alpha] [beta] [eval_rate]",
            args[0]
        );
        eprintln!(
            "Example: {} tsplib/berlin52.tsp 200 25 1.0 4.0 0.2",
            args[0]
        );
        std::process::exit(1);
    }
    let file_path = &args[1];
    println!("Parsing TSP file: {}", file_path);
    let instance = match parse_tsp_file(file_path) {
        Ok(inst) => {
            println!("Successfully parsed: {}", inst.name);
            println!(" Dimension: {}", inst.dimension);
            println!(" Edge Weight Type: {:?}", inst.edge_weight_type);
            println!(" TSP Type: {}", inst.tsp_type);
            println!(" Comment {}", inst.comment);
            if inst.dimension == 0 {
                eprintln!("Error: Problem dimension is 0. Cannot solve.");
                std::process::exit(1);
            }
            inst
        }
        Err(e) => {
            eprintln!("Error parsing TSPLIB file: {}", e);
            std::process::exit(1);
        }
    };

    let mut aco_config = ACOConfig::default();
    if args.len() > 2 {
        aco_config.num_iterations = args[2].parse().unwrap_or(aco_config.num_iterations);
    }
    if args.len() > 3 {
        aco_config.num_ants = args[3].parse().unwrap_or(aco_config.num_ants);
    }
    if args.len() > 4 {
        aco_config.alpha = args[4].parse().unwrap_or(aco_config.alpha);
    }
    if args.len() > 5 {
        aco_config.beta = args[5].parse().unwrap_or(aco_config.beta);
    }
    if args.len() > 6 {
        aco_config.evap_rate = args[6].parse().unwrap_or(aco_config.evap_rate);
    }

    if aco_config.num_ants == 0 {
        aco_config.num_ants = 1;
    }

    println!("\nACO Configuration:");
    println!(" Iterations: {}", aco_config.num_iterations);
    println!(" Number of Ants: {}", aco_config.num_ants);
    println!(" Alpha (pheromone influence): {:.2}", aco_config.alpha);
    println!(" Beta (heuristic influence): {:.2}", aco_config.beta);
    println!(" Evaporation Rate (rho): {:.2}", aco_config.evap_rate);
    println!(" Q Value (pheromone deposit): {:.2}", aco_config.q_val);
    println!(" Initial Pheromone: {:.2}", aco_config.initial_pheromone);
    println!(" Elitist Weight: {:.2}", aco_config.elitist_weight);

    println!("\nStarting ACO to solve TSP for {}...", instance.name);
    let start_time = Instant::now();
    let (best_tour_indices, best_tour_length) = solve_tsp_aco(&instance, &aco_config);
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

    let solutions_file_path = "../tsplib/solutions";
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
}
