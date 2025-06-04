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

        let mut config = Config::default();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-n" | "--ants" => {
                    config.num_ants = args
                        .next()
                        .ok_or("Missing value for --ants")?
                        .parse()
                        .map_err(|_| "Invalid number for --ants")?
                }
                "-i" | "--iters" => {
                    config.num_iters = args
                        .next()
                        .ok_or("Missing value for --iters")?
                        .parse()
                        .map_err(|_| "Invalid number for --iters")?
                }
                "-a" | "--alpha" => {
                    config.alpha = args
                        .next()
                        .ok_or("Missing value for --alpha")?
                        .parse()
                        .map_err(|_| "Invalid number for --alpha")?
                }
                "-b" | "--beta" => {
                    config.beta = args
                        .next()
                        .ok_or("Missing value for --beta")?
                        .parse()
                        .map_err(|_| "Invalid number for --beta")?
                }
                "-e" | "--evap-rate" => {
                    config.evap_rate = args
                        .next()
                        .ok_or("Missing value for --evap-rate")?
                        .parse()
                        .map_err(|_| "Invalid number for --evap-rate")?
                }
                "-q" | "--q-val" => {
                    config.q_val = args
                        .next()
                        .ok_or("Missing value for --q-val")?
                        .parse()
                        .map_err(|_| "Invalid number for --q-val")?
                }
                "-p" | "--init-pheromone" => {
                    config.init_pheromone = args
                        .next()
                        .ok_or("Missing value for --init-pheromone")?
                        .parse()
                        .map_err(|_| "Invalid number for --init-pheromone")?
                }
                "-w" | "--elitist-weight" => {
                    config.elitist_weight = args
                        .next()
                        .ok_or("Missing value for --elitist-weight")?
                        .parse()
                        .map_err(|_| "Invalid number for --elitist-weight")?
                }
                "-m" | "--min-pheromone-val" => {
                    config.min_pheromone_val = args
                        .next()
                        .ok_or("Missing value for --min-pheromone-val")?
                        .parse()
                        .map_err(|_| "Invalid number for --min-pheromone-val")?
                }
                _ if config.file_path.is_none() && !arg.starts_with('-') => {
                    config.file_path = Some(arg)
                }
                _ => return Err("Invalid option or unexpected argument"),
            }
        }
        if config.file_path.is_none() {
            return Err("TSPLIB file path not provided");
        }

        Ok(config)
    }
}
