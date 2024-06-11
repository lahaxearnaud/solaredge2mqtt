use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub pv_to_load: f64,
	pub load_to_grid: f64,
	pub grid_to_load: f64,
    pub pv: f64,
	pub load: f64,
	pub grid: f64,
}
