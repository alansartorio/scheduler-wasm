use js_sys::Array;
use scheduler::{
    models::{Code, Subject, SubjectCommision},
    option_generator::{
        filters::{ChoiceIterator, CreditCount, SubjectCount},
        generate,
    },
};
use wasm_bindgen::prelude::*;

use std::{
    cell::RefCell,
    collections::HashSet,
    iter::FromIterator,
    ops::{Bound, RangeBounds},
    sync::Arc,
};

use crate::{commissions::Commissions, serializer, CollisionExceptions, StringArray};

fn parse_codes(codes: impl IntoIterator<Item = String>) -> impl Iterator<Item = Code> {
    codes.into_iter().map(|c| c.parse().unwrap())
}

impl From<CollisionExceptions> for Vec<((String, String), (String, String))> {
    fn from(ce: CollisionExceptions) -> Self {
        Array::from(&ce)
            .iter()
            .map(|e| {
                let exception: Array = e.into();
                assert_eq!(exception.length(), 2);
                let mut exception = exception.iter().map(|pair| {
                    let commission: Array = pair.into();
                    assert_eq!(commission.length(), 2);
                    let (sub_code, com_name) = (
                        commission.get(0).as_string().unwrap(),
                        commission.get(1).as_string().unwrap(),
                    );
                    let sub_code = sub_code.parse().unwrap();
                    (sub_code, com_name)
                });
                (exception.next().unwrap(), exception.next().unwrap())
            })
            .collect()
    }
}

struct OptionalBound<Idx: PartialOrd>(Option<Idx>);

impl<Idx: PartialOrd> OptionalBound<Idx> {
    fn to_bound(&self) -> Bound<&Idx> {
        match self.0.as_ref() {
            Some(bound) => std::ops::Bound::Included(&bound),
            None => std::ops::Bound::Unbounded,
        }
    }
}

struct OptionallyBoundRange<Idx: PartialOrd> {
    start_bound: OptionalBound<Idx>,
    end_bound: OptionalBound<Idx>,
}
impl<Idx: PartialOrd> OptionallyBoundRange<Idx> {
    fn new(start_bound: Option<Idx>, end_bound: Option<Idx>) -> Self {
        Self {
            start_bound: OptionalBound(start_bound),
            end_bound: OptionalBound(end_bound),
        }
    }
}
impl<Idx: PartialOrd> RangeBounds<Idx> for OptionallyBoundRange<Idx> {
    fn start_bound(&self) -> Bound<&Idx> {
        (&self.start_bound).to_bound()
    }
    fn end_bound(&self) -> Bound<&Idx> {
        self.end_bound.to_bound()
    }
}

#[wasm_bindgen(typescript_custom_section)]
const IOPTION: &'static str = r#"
export type DaysOfTheWeek = "monday" | "tuesday" | "wednesday" | "thursday" | "friday" | "saturday" | "sunday";

export interface Time {
    hour: number,
    minutes: number,
}

export interface Choice {
    subjects: Map<String, {
            name: string,
            credits: number,
            commissions: string[],
    }>,
    week: Map<DaysOfTheWeek, {
            subject: string,
            building: string[],
            span: {
                start: Time,
                end: Time,
            },
    }[]>,
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Choice")]
    pub type Choice;
}

#[wasm_bindgen]
pub struct ChoiceGenerator {
    iter: Box<dyn Iterator<Item = Vec<Option<SubjectCommision>>>>,
}

#[wasm_bindgen]
impl ChoiceGenerator {
    pub fn next_choice(&mut self) -> Choice {
        if let Some(choice) = self.iter.next() {
            let commissions: Vec<_> = choice.into_iter().flatten().collect();
            serde_wasm_bindgen::to_value::<serializer::OptionInfo>(&commissions.into())
                .unwrap()
                .into()
        } else {
            JsValue::null().into()
        }
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct GeneratorBuilder {
    subjects: Commissions,
    mandatory: Vec<Arc<RefCell<Subject>>>,
    optional: Vec<Arc<RefCell<Subject>>>,
    collision_exceptions: HashSet<((Code, SubjectCommision), (Code, SubjectCommision))>,
    min_credit_count: Option<u32>,
    max_credit_count: Option<u32>,
    min_subject_count: Option<u32>,
    max_subject_count: Option<u32>,
}

impl GeneratorBuilder {
    pub fn new(subjects: Commissions) -> Self {
        GeneratorBuilder {
            subjects,
            mandatory: vec![],
            optional: vec![],
            collision_exceptions: HashSet::new(),
            min_credit_count: None,
            max_credit_count: None,
            min_subject_count: None,
            max_subject_count: None,
        }
    }
}

#[wasm_bindgen]
impl GeneratorBuilder {
    pub fn set_min_credit_count(mut self, min_credit_count: Option<u32>) -> Self {
        self.min_credit_count = min_credit_count;
        self
    }

    pub fn set_max_credit_count(mut self, max_credit_count: Option<u32>) -> Self {
        self.max_credit_count = max_credit_count;
        self
    }

    pub fn set_min_subject_count(mut self, min_subject_count: Option<u32>) -> Self {
        self.min_subject_count = min_subject_count;
        self
    }

    pub fn set_max_subject_count(mut self, max_subject_count: Option<u32>) -> Self {
        self.max_subject_count = max_subject_count;
        self
    }

    pub fn set_mandatory_codes(mut self, mandatory_codes: StringArray) -> Self {
        self.mandatory = self
            .subjects
            .find_subjects_by_code(parse_codes(Vec::<String>::from(mandatory_codes)).collect())
            .unwrap();
        self
    }

    pub fn set_optional_codes(mut self, optional_codes: StringArray) -> Self {
        self.optional = self
            .subjects
            .find_subjects_by_code(parse_codes(Vec::<String>::from(optional_codes)).collect())
            .unwrap();
        self
    }

    pub fn set_collision_exceptions(mut self, collision_exceptions: CollisionExceptions) -> Self {
        let collision_exceptions: Vec<((String, String), (String, String))> =
            collision_exceptions.into();
        let find_commission = |sub_code: Code, com_name: String| {
            let sub = self
                .subjects
                .find_subject_by_code(sub_code)
                .unwrap_or_else(|| panic!("Coud not find subject {sub_code}"));
            let sub = sub.borrow();
            sub.commissions
                .iter()
                .find(|c| c.names.iter().any(|name| name == &com_name))
                .unwrap_or_else(|| {
                    panic!("Could not find commission {com_name} from subject {sub_code}.")
                })
                .clone()
        };
        self.collision_exceptions = HashSet::from_iter(collision_exceptions.into_iter().map(
            |((sub_a, com_a), (sub_b, com_b))| {
                let code_a = sub_a.parse().unwrap();
                let code_b = sub_b.parse().unwrap();
                (
                    (code_a, find_commission(code_a, com_a)),
                    (code_b, find_commission(code_b, com_b)),
                )
            },
        ));
        self
    }

    pub fn optimize(&self) {
        self.mandatory
            .iter()
            .for_each(|sub| sub.borrow_mut().optimize());
        self.optional
            .iter()
            .for_each(|sub| sub.borrow_mut().optimize());
    }

    pub fn build(self) -> ChoiceGenerator {
        self.optimize();

        let find_commissions = |subjects: Vec<Arc<RefCell<Subject>>>| {
            subjects
                .into_iter()
                .map(|sub| (sub.borrow().code, sub.borrow().commissions.clone()))
                .collect::<Vec<_>>()
        };
        let mandatory = find_commissions(self.mandatory);
        let optional = find_commissions(self.optional);

        ChoiceGenerator {
            iter: Box::new(
                generate(mandatory, optional, self.collision_exceptions)
                    .filter_choices(SubjectCount::new(OptionallyBoundRange::new(
                        self.min_subject_count,
                        self.max_subject_count,
                    )))
                    .filter_choices(CreditCount::new(OptionallyBoundRange::new(
                        self.min_credit_count,
                        self.max_credit_count,
                    ))),
            ),
        }
    }
}
