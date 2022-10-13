use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Deserialize, Serialize)]
pub struct ServerStatusReport {
    pub healthy: bool,
    pub population: ServerPopulation,
}

#[derive(Deserialize, Serialize, Copy, Clone, Eq, PartialEq)]
pub enum ServerPopulation {
    Full,
    Crowded,
    Populated,
    Easy,
}

impl From<ServerPopulation> for u8 {
    fn from(population: ServerPopulation) -> Self {
        match population {
            ServerPopulation::Full => 0,
            ServerPopulation::Crowded => 1,
            ServerPopulation::Populated => 2,
            ServerPopulation::Easy => 3,
        }
    }
}

impl From<f32> for ServerPopulation {
    fn from(value: f32) -> Self {
        if value < 0.25 {
            ServerPopulation::Easy
        } else if value < 0.6 {
            ServerPopulation::Populated
        } else if value < 0.98 {
            ServerPopulation::Crowded
        } else {
            ServerPopulation::Full
        }
    }
}

impl Display for ServerPopulation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            ServerPopulation::Full => write!(f, "Full"),
            ServerPopulation::Crowded => write!(f, "Crowded"),
            ServerPopulation::Populated => write!(f, "Populated"),
            ServerPopulation::Easy => write!(f, "Easy"),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ReserveRequest {
    pub user_id: u32,
    pub username: String,
}

#[derive(Deserialize, Serialize)]
pub enum ReserveResponse {
    Success { token: u32, alive: u64 },
    NotFound,
    Error(String),
}
