use std::collections::HashMap;

use reqwest;

#[tokio::main]
async fn main() {
    let rasp_html = get_resp_html_from_lk().await;
    parse_rasp_html(&rasp_html);

    println!("Расписание полученно");
}

async fn get_resp_html_from_lk() -> String {
    let url = "https://new.vyatsu.ru/account/obr/rasp/?login=yes";

    let mut body = HashMap::new();

    body.insert("AUTH_FORM", "Y");
    body.insert("TYPE", "AUTH");
    body.insert("backurl", "/account;/url/rasp");
    body.insert("USER_LOGIN", "stud146778");
    body.insert("USER_PASSWORD", "e0I7803");
    body.insert("login", "Войти");

    let client = reqwest::Client::builder().build().unwrap();

    let resp = client.post(url).form(&body).send().await.unwrap();

    let response_data = resp.text().await.unwrap();

    response_data
}

fn parse_rasp_html(reps_html: &str) {
    let dom = tl::parse(reps_html, tl::ParserOptions::default()).unwrap();
    let parser = dom.parser();
    let rasp_days = dom.get_elements_by_class_name("day-container");
    for day in rasp_days {
        let day_info = day.get(parser).unwrap().as_tag().unwrap();
        let date_info = day_info
            .query_selector(parser, "b")
            .unwrap()
            .next()
            .unwrap()
            .get(parser)
            .unwrap()
            .as_tag()
            .unwrap()
            .inner_text(parser);

        let mut splited_date_info = date_info.trim().split(" ");
        let day_of_week = splited_date_info.next().unwrap();
        let date = splited_date_info.next().unwrap();

        println!("День недели: {day_of_week}; Дата: {date}");

        let pairs_info = day_info.query_selector(parser, ".day-pair").unwrap();
        for pair in pairs_info {
            let pair_info = pair.get(parser).unwrap().as_tag().unwrap();
            let time_info = pair_info
                .query_selector(parser, "div")
                .unwrap()
                .next()
                .unwrap()
                .get(parser)
                .unwrap()
                .as_tag()
                .unwrap()
                .inner_text(parser);

            let splited_time_info: Vec<&str> = time_info.trim().split(" ").collect();
            let pair_number = splited_time_info[1];
            let time_period = splited_time_info.last().unwrap();

            println!("\nПара: {pair_number}; Время: {time_period}");

            let pair_description = pair_info
                .query_selector(parser, ".pair_desc")
                .unwrap()
                .next()
                .unwrap()
                .get(parser)
                .unwrap()
                .as_tag()
                .unwrap()
                .inner_text(parser);

            let pair_desc_splited: Vec<&str> = pair_description.trim().split(",").collect();
            let pair_name = pair_desc_splited[0].trim();
            let pair_type = pair_desc_splited[1].trim();
            let pair_teacher = pair_desc_splited[2].trim();

            println!(
                "Предмет: {pair_name};\nТип занятия: {pair_type};\nПреподаватель: {pair_teacher}\n"
            );
        }

        println!("\n---------------------------------------------\n")
    }
}
