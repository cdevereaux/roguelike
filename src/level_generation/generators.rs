use bevy::prelude::*;

#[derive(Debug, Resource, Clone, Copy)]
pub enum MapGeneratorSettings {
    Cavern(CavernSettings),
}

#[derive(Debug, Clone, Copy)]
pub struct CavernSettings {
    pub cavern_count: usize,
    pub max_cavern_dist: usize,
    pub walk_count: usize,
    pub walk_len: usize,
}

impl Default for CavernSettings {
    fn default() -> Self {
        CavernSettings {
            cavern_count: 6,
            max_cavern_dist: 70,
            walk_count: 100,
            walk_len: 50,
        }
    }
}

impl Default for MapGeneratorSettings {
    fn default() -> Self {
        MapGeneratorSettings::Cavern(CavernSettings::default())
    }
}
