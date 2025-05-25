use crate::{
    error::JujikError,
    pin::Pin,
    tab::Tab,
    view::{EntitysShowColumn, JujikStyle},
};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

const CONFIG_PATH: &'static str = "./config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub style: JujikStyle,
    pub pins: Vec<Pin>,
    pub tabs: Vec<Tab>,
    pub current_tab_idx: usize,
    pub entitys_show: EntitysShowColumn,
}

impl Config {
    pub fn new(
        style: JujikStyle,
        pins: Vec<Pin>,
        tabs: Vec<Tab>,
        current_tab_idx: usize,
        entitys_show: EntitysShowColumn,
    ) -> Self {
        Self {
            style,
            pins,
            tabs,
            current_tab_idx,
            entitys_show,
        }
    }

    pub fn write(&self) -> Result<(), JujikError> {
        Ok(fs::write(CONFIG_PATH, serde_json::to_string_pretty(self)?)?)
    }

    pub fn load() -> Result<Self, JujikError> {
        Ok(serde_json::from_str(&fs::read_to_string(CONFIG_PATH)?)?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            style: JujikStyle::default(),
            pins: vec![
                Pin::new("Home".to_string(), PathBuf::from("/home/sanart0/")),
                Pin::new(
                    "Test".to_string(),
                    PathBuf::from("/home/sanart0/KPI/4/IPZ-Kursach/jujik/test/"),
                ),
            ],
            tabs: Vec::new(),
            current_tab_idx: 0,
            entitys_show: EntitysShowColumn::default(),
        }
    }
}
