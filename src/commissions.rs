use std::sync::Arc;

use crate::{generator::GeneratorBuilder, SubjectInfo};
use anyhow::{anyhow, Result};
use scheduler::models::{Code, Subject};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Commissions {
    subjects: Arc<Vec<Arc<Subject>>>,
}

impl Commissions {
    pub fn new(subjects: Vec<Arc<Subject>>) -> Self {
        Self {
            subjects: Arc::new(subjects),
        }
    }

    pub fn find_subject_by_code(&self, code: Code) -> Option<Arc<Subject>> {
        self.subjects.iter().find(|s| s.code == code).cloned()
    }

    pub fn find_subjects_by_code(&self, codes: Vec<Code>) -> Result<Vec<Arc<Subject>>> {
        codes
            .into_iter()
            //.map(|c| c.parse().unwrap())
            .map(|code| {
                self.find_subject_by_code(code)
                    .ok_or_else(|| anyhow!("Subject {} was not found", code))
            })
            .collect::<Result<Vec<_>>>()
    }
}

#[wasm_bindgen]
impl Commissions {
    pub fn get_subject_info(&self, code: String) -> Option<SubjectInfo> {
        let code: Code = code.parse().unwrap();
        self.subjects
            .iter()
            .find(|s| s.code == code)
            .map(|s| SubjectInfo {
                code: s.code,
                name: s.name.clone(),
                credits: s.credits,
            })
    }

    pub fn create_generator_builder(&self) -> GeneratorBuilder {
        GeneratorBuilder::new(self.clone())
    }
}
