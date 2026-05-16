use serde::{Deserialize, Serialize};

pub const DEFAULT_CONFIDENCE_THRESHOLD: f32 = 0.8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionConfidence {
    pub section: String,
    pub score: f32,
    pub citation_ids: Vec<String>,
    pub notes: Option<String>,
}

impl SectionConfidence {
    pub fn new(section: impl Into<String>, score: f32) -> Self {
        Self {
            section: section.into(),
            score: score.clamp(0.0, 1.0),
            citation_ids: Vec::new(),
            notes: None,
        }
    }

    #[must_use]
    pub fn with_citations<I, S>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.citation_ids = ids.into_iter().map(Into::into).collect();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceReport {
    pub threshold: f32,
    pub sections: Vec<SectionConfidence>,
}

impl Default for ConfidenceReport {
    fn default() -> Self {
        Self { threshold: DEFAULT_CONFIDENCE_THRESHOLD, sections: Vec::new() }
    }
}

impl ConfidenceReport {
    pub fn passes(&self) -> bool {
        self.sections.iter().all(|s| s.score >= self.threshold)
    }

    pub fn weak_sections(&self) -> Vec<&SectionConfidence> {
        self.sections.iter().filter(|s| s.score < self.threshold).collect()
    }
}
