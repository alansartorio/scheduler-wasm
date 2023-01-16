use crate::{StringArray, SubjectInfo};
use itertools::Itertools;
use js_sys::{Object, Reflect};
use scheduler::json_parser::{CareerPlan, Entry, SubjectEntry};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SubjectPlan {
    data: CareerPlan,
}

impl SubjectPlan {
    pub fn new(data: CareerPlan) -> Self {
        Self { data }
    }
}

fn get_subjects(career_plan: &CareerPlan) -> impl Iterator<Item = &SubjectEntry> {
    career_plan
        .sections
        .iter()
        .flat_map(|s| {
            s.terms
                .iter()
                .flat_map(|t| t.entries.iter())
                .chain(s.without_term.iter())
                .filter_map(|e| {
                    if let Entry::Subject(subject) = e {
                        Some(subject)
                    } else {
                        None
                    }
                })
        })
        .unique_by(|s| s.code)
}

#[wasm_bindgen]
impl SubjectPlan {
    pub fn get_subject_dependencies(&self, code: String) -> Option<StringArray> {
        let code = code.parse().unwrap();
        get_subjects(&self.data).find(|s| s.code == code).map(|s| {
            s.dependencies
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .into()
        })
    }

    pub fn get_subjects(&self) -> StringArray {
        get_subjects(&self.data)
            .map(|s| s.code.to_string())
            .collect::<Vec<_>>()
            .into()
    }

    pub fn get_subject_info(&self, code: String) -> Option<SubjectInfo> {
        let code = code.parse().unwrap();
        get_subjects(&self.data)
            .find(|s| s.code == code)
            .map(|s| SubjectInfo {
                code: s.code,
                name: s.name.clone(),
                credits: s.credits,
            })
    }

    pub fn get_subject_terms(&self, code: String) -> Vec<Object> {
        let code = code.parse().unwrap();
        self.data
            .sections
            .iter()
            .flat_map(|s| {
                s.terms.iter().flat_map(|t| {
                    if t.entries.iter().any(|e| {
                        if let Entry::Subject(subject) = e && subject.code == code {
                            true
                        } else {
                            false
                        }
                    }) {
                        Some(&t.term)
                    } else {
                        None
                    }
                })
            })
            .map(|term| {
                let js_term = Object::new();
                Reflect::set(&js_term, &"year".into(), &term.year.into()).unwrap();
                Reflect::set(&js_term, &"period".into(), &term.period.into()).unwrap();
                js_term
            })
            .collect_vec()
    }
}
