//! Age scale derived from prehistory years

/// Age scale derived from prehistory years
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgeScale {
    PreHistory(u32),  // < 200 years
    Ancient(u32),     // 200-1000 years
    Medieval(u32),    // 1000-3000 years
    Modern(u32),      // 3000+ years
}

impl AgeScale {
    pub fn from_pre_history_years(years: u32) -> Self {
        if years < 200 {
            AgeScale::PreHistory(years)
        } else if years < 1000 {
            AgeScale::Ancient(years)
        } else if years < 3000 {
            AgeScale::Medieval(years)
        } else {
            AgeScale::Modern(years)
        }
    }

    pub fn faction_count(&self) -> u32 {
        match self {
            AgeScale::PreHistory(_) => 2,
            AgeScale::Ancient(_) => 3,
            AgeScale::Medieval(_) => 4,
            AgeScale::Modern(_) => 5,
        }
    }
}

/// Territory with age scale
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Territory {
    pub id: String,
    pub name: String,
    pub pre_history_years: u32,
}

impl Territory {
    pub fn new(name: String, pre_history_years: u32) -> Self {
        let age_scale = AgeScale::from_pre_history_years(pre_history_years);
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            pre_history_years,
        }
    }

    pub fn age_scale(&self) -> AgeScale {
        AgeScale::from_pre_history_years(self.pre_history_years)
    }
}
