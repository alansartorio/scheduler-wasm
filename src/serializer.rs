use enum_map::EnumMap;
use itertools::Itertools;
use scheduler::{
    json_parser::Code,
    models::{Combinable, DaysOfTheWeek, SubjectCommision, Week},
};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone, Serialize)]
struct Subject {
    name: String,
    credits: u32,
    commissions: Vec<String>,
}

#[derive(Clone, Copy, Serialize)]
struct Time {
    hour: u8,
    minutes: u8,
}

impl From<scheduler::models::Time> for Time {
    fn from(scheduler::models::Time { hour, minutes }: scheduler::models::Time) -> Self {
        Self { hour, minutes }
    }
}

#[derive(Clone, Copy, Serialize)]
struct Span {
    start: Time,
    end: Time,
}

impl From<scheduler::models::Span> for Span {
    fn from(scheduler::models::Span { start, end }: scheduler::models::Span) -> Self {
        Self {
            start: start.into(),
            end: end.into(),
        }
    }
}

#[derive(Clone, Serialize)]
struct Task {
    subject: Code,
    buildings: Vec<String>,
    //classroom: String,
    span: Span,
}

#[derive(Clone, Serialize)]
pub struct OptionInfo {
    subjects: HashMap<Code, Subject>,
    week: EnumMap<DaysOfTheWeek, Vec<Task>>,
}

impl From<Vec<SubjectCommision>> for OptionInfo {
    fn from(commissions: Vec<SubjectCommision>) -> Self {
        let subjects: HashMap<_, _> = commissions
            .iter()
            .map(|c| {
                let s = c.subject.upgrade().unwrap();
                let s = s.borrow();
                (
                    s.code,
                    Subject {
                        commissions: c.names.clone(),
                        name: s.name.clone(),
                        credits: s.credits as u32,
                    },
                )
            })
            .collect();

        let week = commissions
            .iter()
            .map(|c| &c.schedule)
            .fold(Week::empty(), |a, b| Week::combine(&a, b))
            .days
            .map(|_, scheduler::models::Day { tasks, .. }| {
                tasks
                    .iter()
                    .map(|task| Task {
                        subject: task.info.subject.upgrade().unwrap().borrow().code,
                        span: task.span.into(),
                        buildings: task
                            .info
                            .buildings
                            .iter()
                            .map(|b| b.name.clone())
                            .collect_vec(),
                    })
                    .collect()
            });

        Self { subjects, week }
    }
}
