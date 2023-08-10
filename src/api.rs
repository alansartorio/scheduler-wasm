use scheduler::{json_parser::CareerPlan, loaders::json_loader};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Request, RequestInit, RequestMode, Response};

use crate::{commissions::Commissions, plan::SubjectPlan, Semester};

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

    pub async fn get_commissions_from_api(&self, year: u32, semester: Semester) -> Commissions {
        let url = format!(
            "{}/commissions/GRADUATE-{}-{}.json",
            self.url_base,
            year,
            match semester {
                Semester::First => "FirstSemester",
                Semester::Second => "SecondSemester",
            }
        );

        let body = fetch(&url).await;

        let data = json_loader::load_from_string(&body).unwrap();

        Commissions::new(data)
    }

    pub async fn get_plan_from_api(&self, plan: &str) -> SubjectPlan {
        let url = format!("{}/plan/{}.json", self.url_base, plan);

        let body = fetch(&url).await;

        SubjectPlan::new(serde_json::from_str::<CareerPlan>(&body).unwrap())
    }
}
