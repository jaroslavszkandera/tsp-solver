#[derive(Debug, Clone)]
pub struct Config {
    pub file_path: Option<String>,
    pub num_iters: usize,
    pub num_ants: usize,
    pub alpha: f64,     // Pheromone influence
    pub beta: f64,      // Heuristic influence
    pub evap_rate: f64, // Rho
    pub q_val: f64,     // Pheromone deposit amount scaling factor
    pub init_pheromone: f64,
    pub elitist_weight: f64, // Weight for the elitist ant's pheromone deposit
    pub min_pheromone_val: f64, // Minimum pheromone value
}

impl Default for Config {
    fn default() -> Self {
        Config {
            file_path: None,
            num_iters: 1000,
            num_ants: 50,
            alpha: 1.0,
            beta: 3.0,
            evap_rate: 0.1,
            q_val: 100.0,
            init_pheromone: 0.1,
            elitist_weight: 1.0, // e.g. 1 means global best adds pheromone like one ant
            min_pheromone_val: 1e-5,
        }
    }
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let mut file_path: Option<String> = None;
        let mut num_ants = 10;
        let mut num_iters = 100;
        let mut alpha = 0.5;
        let mut beta = 2.0;
        let mut evap_rate = 0.5;
        let mut q_val = 100.0;
        let mut init_pheromone = 0.2;
        let mut elitist_weight = 1.0;
        let mut min_pheromone_val = 1e-5;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-n" | "--ants" => {
                    num_ants = args
                        .next()
                        .ok_or("Missing value for --ants")?
                        .parse()
                        .map_err(|_| "Invalid number for --ants")?
                }
                "-i" | "--iters" => {
                    num_iters = args
                        .next()
                        .ok_or("Missing value for --iters")?
                        .parse()
                        .map_err(|_| "Invalid number for --iters")?
                }
                "-a" | "--alpha" => {
                    alpha = args
                        .next()
                        .ok_or("Missing value for --alpha")?
                        .parse()
                        .map_err(|_| "Invalid number for --alpha")?
                }
                "-b" | "--beta" => {
                    beta = args
                        .next()
                        .ok_or("Missing value for --beta")?
                        .parse()
                        .map_err(|_| "Invalid number for --beta")?
                }
                "-e" | "--evap-rate" => {
                    evap_rate = args
                        .next()
                        .ok_or("Missing value for --evap-rate")?
                        .parse()
                        .map_err(|_| "Invalid number for --evap-rate")?
                }
                "-q" | "--q-val" => {
                    q_val = args
                        .next()
                        .ok_or("Missing value for --q-val")?
                        .parse()
                        .map_err(|_| "Invalid number for --q-val")?
                }
                "-p" | "--init-pheromone" => {
                    init_pheromone = args
                        .next()
                        .ok_or("Missing value for --init-pheromone")?
                        .parse()
                        .map_err(|_| "Invalid number for --init-pheromone")?
                }
                "-w" | "--elitist-weight" => {
                    elitist_weight = args
                        .next()
                        .ok_or("Missing value for --elitist-weight")?
                        .parse()
                        .map_err(|_| "Invalid number for --elitist-weight")?
                }
                "-m" | "--min-pheromone-val" => {
                    min_pheromone_val = args
                        .next()
                        .ok_or("Missing value for --min-pheromone-val")?
                        .parse()
                        .map_err(|_| "Invalid number for --min-pheromone-val")?
                }
                _ if file_path.is_none() && !arg.starts_with('-') => file_path = Some(arg),
                _ => return Err("Invalid option or unexpected argument"),
            }
        }
        file_path = Some(file_path.ok_or("TSPLIB file path not provided")?);

        Ok(Config {
            file_path,
            num_ants,
            num_iters,
            alpha,
            beta,
            evap_rate,
            q_val,
            init_pheromone,
            elitist_weight,
            min_pheromone_val,
        })
    }
}
