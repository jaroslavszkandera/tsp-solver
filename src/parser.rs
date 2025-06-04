use std::fs::File as StdFile;
use std::io::{BufRead, BufReader as StdBufReader};

fn calc_euc_2d_dist(n1: &Node, n2: &Node) -> f64 {
    let dx = n1.x - n2.x;
    let dy = n1.y - n2.y;
    ((dx * dx + dy * dy).sqrt()).round()
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
