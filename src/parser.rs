use std::f64::consts::PI;
use std::fs::File as StdFile;
use std::io::{BufRead, BufReader as StdBufReader};

fn to_radians(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

fn calc_euc_2d_dist(n1: &Node, n2: &Node) -> f64 {
    let dx = n1.x - n2.x;
    let dy = n1.y - n2.y;
    (dx * dx + dy * dy).sqrt()
}

fn calc_ceil_2d_dist(n1: &Node, n2: &Node) -> f64 {
    let dx = n1.x - n2.x;
    let dy = n1.y - n2.y;
    ((dx * dx + dy * dy).sqrt()).ceil()
}

fn calc_geo_dist(n1: &Node, n2: &Node) -> f64 {
    const RRR: f64 = 6378.388; // Earth radius in km

    // n.x is longitude, n.y is latitude
    let lon1_rad = to_radians(n1.x);
    let lat1_rad = to_radians(n1.y);
    let lon2_rad = to_radians(n2.x);
    let lat2_rad = to_radians(n2.y);

    let q1 = (lon1_rad - lon2_rad).cos();
    let q2 = (lat1_rad - lat2_rad).cos();
    let q3 = (lat1_rad + lat2_rad).cos();

    let distance = RRR * (0.5 * ((1.0 + q1) * q2 - (1.0 - q1) * q3)).acos() + 1.0;
    distance
}

fn calc_att_dist(n1: &Node, n2: &Node) -> f64 {
    let dx = n1.x - n2.x;
    let dy = n1.y - n2.y;
    let rij = ((dx * dx + dy * dy) / 10.0).sqrt();
    let tij = rij.round();
    if tij < rij { tij + 1.0 } else { tij }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeWeightType {
    Euc2D,    // berlin52
    Ceil2D,   // dsj1000
    Geo,      // ulysses16
    Att,      // att48
    Explicit, // gr17, bayg29, bays29
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeWeightFormat {
    Function,
    FullMatrix,
    UpperRow,
    LowerRow,
    LowerDiagRow,
    UpperDiagRow,
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
    pub edge_weight_format: Option<EdgeWeightFormat>,
    pub node_coords: Option<Vec<Node>>,
    pub dist_matrix: Vec<Vec<f64>>,
}

impl TspInstance {
    #[allow(dead_code)]
    pub fn get_dist(&self, node1_idx: usize, node2_idx: usize) -> f64 {
        if node1_idx >= self.dimension || node2_idx >= self.dimension {
            panic!(
                "Node index out of bounds ({} or {} for dimension {})",
                node1_idx, node2_idx, self.dimension
            );
        }
        self.dist_matrix[node1_idx][node2_idx]
    }
}

#[derive(PartialEq, Debug)]
enum ParsingSection {
    Header,
    NodeCoordSection,
    EdgeWeightSection,
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
    let mut edge_weight_format_str: Option<String> = None;
    let mut node_coords_vec: Vec<Node> = Vec::new();
    let mut explicit_weights_data: Vec<f64> = Vec::new();

    let mut current_section = ParsingSection::Header;
    let mut current_line_num = 0;

    for line_result in reader.lines() {
        current_line_num += 1;
        let line = line_result
            .map_err(|e| format!("Error reading line {}: {}", current_line_num, e))?
            .trim()
            .to_string();

        if line == "EOF" {
            break;
        }
        if line.is_empty() {
            continue;
        }

        if line == "NODE_COORD_SECTION" {
            current_section = ParsingSection::NodeCoordSection;
            continue;
        } else if line == "EDGE_WEIGHT_SECTION" {
            current_section = ParsingSection::EdgeWeightSection;
            continue;
        } else if line == "DISPLAY_DATA_SECTION" || line == "TOUR_SECTION" {
            if current_section == ParsingSection::NodeCoordSection
                && node_coords_vec.len() != dimension
                && dimension > 0
            {
                return Err(format!(
                    "L{}: Started new section '{}' before all node coordinates were read. Expected {}, got {}.",
                    current_line_num,
                    line,
                    dimension,
                    node_coords_vec.len()
                ));
            }
            current_section = ParsingSection::Header;
            continue;
        }

        match current_section {
            ParsingSection::Header => {
                let parts: Vec<&str> = line.splitn(2, ':').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    let key = parts[0];
                    let value = parts[1];
                    match key {
                        "NAME" => name = value.to_string(),
                        "TYPE" => tsp_type = value.to_string(),
                        "COMMENT" => {
                            if !comment.is_empty() {
                                comment.push_str("; ");
                            }
                            comment.push_str(value);
                        }
                        "DIMENSION" => {
                            dimension = value.parse::<usize>().map_err(|e| {
                                format!(
                                    "L{}: Invalid dimension: {} on line '{}'",
                                    current_line_num, e, line
                                )
                            })?;
                        }
                        "EDGE_WEIGHT_TYPE" => edge_weight_type_str = value.to_string(),
                        "EDGE_WEIGHT_FORMAT" => edge_weight_format_str = Some(value.to_string()),
                        _ => {} // Ignore other keywords
                    }
                }
            }
            ParsingSection::NodeCoordSection => {
                if node_coords_vec.len() == dimension {
                    return Err(format!(
                        "L{}: Unexpected data after all node coordinates were read: '{}'. Expected {} nodes.",
                        current_line_num, line, dimension
                    ));
                }
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
                            "L{}: Invalid x/lon coord: {} on line '{}'",
                            current_line_num, e, line
                        )
                    })?;
                    let y = parts[2].parse::<f64>().map_err(|e| {
                        format!(
                            "L{}: Invalid y/lat coord: {} on line '{}'",
                            current_line_num, e, line
                        )
                    })?;
                    node_coords_vec.push(Node { id, x, y });
                } else {
                    return Err(format!(
                        "L{}: Malformed node coord line (expected id x y): {}",
                        current_line_num, line
                    ));
                }
            }
            ParsingSection::EdgeWeightSection => {
                let nums_str: Vec<&str> = line.split_whitespace().collect();
                for s_num in nums_str {
                    if !s_num.is_empty() {
                        explicit_weights_data.push(s_num.parse::<f64>().map_err(|e| {
                            format!(
                                "L{}: Invalid edge weight number: '{}', error: {}",
                                current_line_num, s_num, e
                            )
                        })?);
                    }
                }
            }
        }
    }

    if dimension == 0 {
        return Err("DIMENSION not found or is zero.".to_string());
    }

    let ewt = match edge_weight_type_str.to_uppercase().as_str() {
        "EUC_2D" => EdgeWeightType::Euc2D,
        "GEO" => EdgeWeightType::Geo,
        "ATT" => EdgeWeightType::Att,
        "EXPLICIT" => EdgeWeightType::Explicit,
        "CEIL_2D" => EdgeWeightType::Ceil2D,
        s => EdgeWeightType::Unknown(s.to_string()),
    };

    let ewf = match ewt {
        EdgeWeightType::Explicit => {
            match edge_weight_format_str.as_deref().map(|s| s.to_uppercase()) {
                Some(s) if s == "FULL_MATRIX" => Some(EdgeWeightFormat::FullMatrix),
                Some(s) if s == "UPPER_ROW" => Some(EdgeWeightFormat::UpperRow),
                Some(s) if s == "LOWER_DIAG_ROW" => Some(EdgeWeightFormat::LowerDiagRow),
                // TODO: Add other formats like
                Some(s) => Some(EdgeWeightFormat::Unknown(s)),
                None => return Err("EDGE_WEIGHT_FORMAT missing for EXPLICIT type.".to_string()),
            }
        }
        _ => edge_weight_format_str
            .clone()
            .map(EdgeWeightFormat::Unknown),
    };

    match ewt {
        EdgeWeightType::Euc2D
        | EdgeWeightType::Geo
        | EdgeWeightType::Att
        | EdgeWeightType::Ceil2D => {
            if node_coords_vec.len() != dimension {
                return Err(format!(
                    "Mismatch: DIMENSION ({}) vs found node coordinates ({}). Type: {:?}",
                    dimension,
                    node_coords_vec.len(),
                    ewt
                ));
            }
            if node_coords_vec.is_empty() && dimension > 0 {
                return Err(format!(
                    "Node coordinates are required for edge weight type {:?} but none were found.",
                    ewt
                ));
            }
        }
        EdgeWeightType::Explicit => {
            if ewf.is_none() || matches!(ewf, Some(EdgeWeightFormat::Unknown(_))) {
                return Err(format!(
                    "Unsupported or missing EDGE_WEIGHT_FORMAT for EXPLICIT type: {:?}",
                    edge_weight_format_str
                ));
            }
        }
        EdgeWeightType::Unknown(ref s) => return Err(format!("Unknown edge weight type: {}", s)),
    }

    let mut dist_matrix = vec![vec![0.0; dimension]; dimension];

    match ewt {
        EdgeWeightType::Euc2D
        | EdgeWeightType::Ceil2D
        | EdgeWeightType::Geo
        | EdgeWeightType::Att => {
            let coords = &node_coords_vec;
            if coords.len() != dimension {
                return Err(format!(
                    "Dimension mismatch: expected {} nodes, found {} in coordinates for type {:?}",
                    dimension,
                    coords.len(),
                    ewt
                ));
            }
            for i in 0..dimension {
                for j in 0..dimension {
                    if i == j {
                        dist_matrix[i][j] = 0.0;
                        continue;
                    }
                    let n1 = &coords[i];
                    let n2 = &coords[j];
                    dist_matrix[i][j] = match ewt {
                        EdgeWeightType::Euc2D => calc_euc_2d_dist(n1, n2),
                        EdgeWeightType::Ceil2D => calc_ceil_2d_dist(n1, n2),
                        EdgeWeightType::Geo => calc_geo_dist(n1, n2),
                        EdgeWeightType::Att => calc_att_dist(n1, n2),
                        _ => unreachable!(),
                    };
                }
            }
        }
        EdgeWeightType::Explicit => match ewf.as_ref().unwrap() {
            EdgeWeightFormat::FullMatrix => {
                if explicit_weights_data.len() != dimension * dimension {
                    return Err(format!(
                        "EXPLICIT FULL_MATRIX: Expected {} weights ({}*{}_DIM), got {}.",
                        dimension * dimension,
                        dimension,
                        dimension,
                        explicit_weights_data.len()
                    ));
                }
                let mut k = 0;
                for i in 0..dimension {
                    for j in 0..dimension {
                        dist_matrix[i][j] = explicit_weights_data[k];
                        k += 1;
                    }
                }
            }
            EdgeWeightFormat::UpperRow => {
                let expected_weights = dimension * (dimension - 1) / 2;
                if explicit_weights_data.len() != expected_weights {
                    return Err(format!(
                        "EXPLICIT UPPER_ROW: Expected {} weights, got {}.",
                        expected_weights,
                        explicit_weights_data.len()
                    ));
                }
                let mut k = 0;
                for i in 0..dimension {
                    for j in (i + 1)..dimension {
                        dist_matrix[i][j] = explicit_weights_data[k];
                        dist_matrix[j][i] = explicit_weights_data[k];
                        k += 1;
                    }
                }
            }
            EdgeWeightFormat::LowerDiagRow => {
                let expected_weights = dimension * (dimension + 1) / 2;
                if explicit_weights_data.len() != expected_weights {
                    return Err(format!(
                        "EXPLICIT LOWER_DIAG_ROW: Expected {} weights, got {}.",
                        expected_weights,
                        explicit_weights_data.len()
                    ));
                }
                let mut k = 0;
                for i in 0..dimension {
                    for j in 0..=i {
                        dist_matrix[i][j] = explicit_weights_data[k];
                        if i != j {
                            dist_matrix[j][i] = explicit_weights_data[k];
                        }
                        k += 1;
                    }
                }
            }
            EdgeWeightFormat::Unknown(s) => {
                return Err(format!("Unsupported EXPLICIT format: {}", s));
            }
            _ => return Err("Unhandled EXPLICIT format during matrix population.".to_string()),
        },
        EdgeWeightType::Unknown(ref s) => {
            return Err(format!(
                "Cannot populate distance matrix for unknown edge weight type: {}",
                s
            ));
        }
    }

    Ok(TspInstance {
        name,
        tsp_type,
        comment,
        dimension,
        edge_weight_type: ewt,
        edge_weight_format: ewf,
        node_coords: if node_coords_vec.is_empty() {
            None
        } else {
            Some(node_coords_vec)
        },
        dist_matrix,
    })
}
