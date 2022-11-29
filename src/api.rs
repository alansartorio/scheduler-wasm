use std::sync::{Arc, Mutex};

use scheduler::{
    json_parser::{CareerPlan, Code, Entry, SubjectEntry},
    loaders::json_loader,
    models::Subject,
};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Request, RequestInit, RequestMode, Response};

use crate::{Semester, SubjectInfo};

async fn fetch(url: &str) -> String {
    let mut opts = RequestInit::new();
    opts.method("GET").mode(RequestMode::Cors);
    let window = window().unwrap();

    let request = Request::new_with_str_and_init(url, &opts).unwrap();
    let resp = JsFuture::from(window.fetch_with_request(&request))
        .await
        .unwrap();

    let resp: Response = resp.dyn_into().unwrap();
    JsFuture::from(resp.text().unwrap())
        .await
        .unwrap()
        .as_string()
        .unwrap()
}

pub static SUBJECTS: Mutex<Vec<Arc<Subject>>> = Mutex::new(vec![]);

#[wasm_bindgen]
pub struct SubjectPlan {
    data: CareerPlan,
}

fn get_subjects(career_plan: &CareerPlan) -> impl Iterator<Item = &SubjectEntry> {
    career_plan.sections.iter().flat_map(|s| {
        s.terms.iter().flat_map(|t| {
            t.term.iter().flat_map(|t| {
                t.entries.entry.iter().filter_map(|e| {
                    if let Entry::Subject(subject) = e {
                        Some(subject)
                    } else {
                        None
                    }
                })
            })
        })
    })
}

impl SubjectPlan {
    pub fn get_subject_dependencies(&self, code: Code) -> Option<Vec<Code>> {
        get_subjects(&self.data)
            .find(|s| s.code == code)
            .map(|s| s.dependencies.0.clone())
    }

    pub fn get_subjects(&self) -> Vec<Code> {
        get_subjects(&self.data).map(|s| s.code).collect()
    }

    pub fn get_subject_info(&self, code: Code) -> Option<SubjectInfo> {
        get_subjects(&self.data)
            .find(|s| s.code == code)
            .map(|s| SubjectInfo {
                code: s.code,
                name: s.name.clone(),
                credits: s.credits,
            })
    }
}

#[wasm_bindgen]
pub struct Api {
    url_base: String,
}

#[wasm_bindgen]
impl Api {
    #[wasm_bindgen(constructor)]
    pub fn new(url_base: String) -> Self {
        Self { url_base }
    }

    pub async fn load_subjects_from_api(&self, year: u32, semester: Semester) {
        let url = format!(
            "{}/commissions?year={}&period={}",
            self.url_base,
            year,
            match semester {
                Semester::First => "FirstSemester",
                Semester::Second => "SecondSemester",
            }
        );

        let body = fetch(&url).await;

        let data = json_loader::load_from_string(&body).unwrap();
        let mut subjects = SUBJECTS.lock().unwrap();
        *subjects = data;
    }

    pub async fn get_plan_from_api(&self, plan: &str) -> SubjectPlan {
        let url = format!("{}/plan?name={}", self.url_base, plan);

        let body = fetch(&url).await;

        SubjectPlan {
            data: serde_json::from_str::<CareerPlan>(&body).unwrap(),
        }
    }
}
