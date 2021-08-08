use serde::{Deserialize, Serialize};

use crate::schema::flowers;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
pub struct Flower {
    pub flw_id: String,
    pub flw_source: String,
    pub flw_name: String,
    pub flw_img: String,
    pub flw_family: String,
    pub flw_season: String,
    pub flw_desc: String,
    pub flw_site_chars: String,
    pub flw_plant_traits: String,
    pub flw_special_cons: String,
    pub flw_growing_infos: String,
    pub flw_varieties: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewFlower {
    pub flw_source: String,
    pub flw_name: String,
    pub flw_img: String,
    pub flw_family: String,
    pub flw_season: String,
    pub flw_desc: String,
    pub flw_site_chars: String,
    pub flw_plant_traits: String,
    pub flw_special_cons: String,
    pub flw_growing_infos: String,
    pub flw_varieties: String,
}