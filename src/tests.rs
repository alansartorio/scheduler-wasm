#![cfg(target_arch = "wasm32")]

use crate::{get_subject_info, load_from_api};

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_get_from_api() {
    //let server = MockServer::start();

    //let api_mock = server.mock(|when, then| {
    //when.method(GET)
    //.path("/api")
    //.query_param("year", "2022")
    //.query_param("period", "SecondSemester");
    //then.status(200)
    //.header("content-type", "application/json")
    //.body(
    //r#"
    //{
    //"courseCommissions": {
    //"courseCommission": [
    //{
    //"subjectCode": "00.00",
    //"subjectName": "Test subject",
    //"subjectType": "NORMAL",
    //"courseStart": "01/01/2001",
    //"courseEnd": "01/01/2001",
    //"commissionName": "TEST",
    //"commissionId": "12345",
    //"quota": "30",
    //"enrolledStudents": "1",
    //"courseCommissionTimes": [],
    //}
    //]
    //}
    //}
    //"#,
    //);
    //});

    //load_from_api(server.url("/api"), 2022, crate::Semester::Second).await;
    load_from_api(
        "http://localhost/api".to_owned(),
        2022,
        crate::Semester::Second,
    )
    .await;

    let subject = get_subject_info("00.00".to_owned()).unwrap();

    assert_eq!(subject.name, "Test subject");

    //api_mock.assert();
}
