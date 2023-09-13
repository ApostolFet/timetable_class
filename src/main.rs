use std::collections::HashMap;
use std::fs::{read_to_string, File};

use chrono::prelude::*;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use teloxide::prelude::*;
use teloxide::types::ParseMode;

#[tokio::main]
async fn main() {
    let rasp_html = get_resp_html_from_lk().await;
    let rasp = parse_rasp_html(&rasp_html);

    let file = File::create("rasp.json").unwrap();
    serde_json::to_writer_pretty(file, &rasp).unwrap();

    let input_rasp = read_to_string("rasp.json").unwrap();

    let all_day_rasp: Vec<StudentDay> = serde_json::from_str(&input_rasp).unwrap();
    let today_rasp = find_today_rasp(all_day_rasp);
    if let Some(today_rasp) = today_rasp {
        // Обработка варианта Some
        println!("Отправляем сообщение");
        send_rasp_to_tg(today_rasp).await;
    }
}

async fn send_rasp_to_tg(student_day: StudentDay) {
    let bot = Bot::from_env();

    // Specify the chat ID and the message text
    let chat_id = "-4058835081"; // Replace with the actual chat ID
    let mut message_text = format!(
        "Внимание сегодня по расписанию есть пары!!!\nДата: {} ({})\n",
        student_day.date, student_day.day_of_week
    );
    for pair in student_day.pairs {
        let pair_time = format!("\nПара №{}; Время: {}\n", pair.number, pair.time_period);
        let pair_name = format!("Предмет: {}\n", pair.name);
        let pair_type = format!("Тип: {}\n", pair.pair_type);
        let pair_teacher = format!("Преподаватель: {}\n", pair.teacher);
        let pair_url = format!(
            "<a href='{}'>Занаятие удаленно в Teams</a>\n",
            pair.teams_url_pair
        );

        message_text =
            message_text + &pair_time + &pair_name + &pair_type + &pair_teacher + &pair_url;
    }
    println!("{message_text}");
    let _ = bot
        .send_message(chat_id.to_string(), message_text)
        .parse_mode(ParseMode::Html)
        .send()
        .await
        .unwrap();
}

fn find_today_rasp(all_day_rasp: Vec<StudentDay>) -> Option<StudentDay> {
    let local: DateTime<Local> = Local::now();
    let today = local.format("%d.%m.%Y").to_string();

    for student_day in all_day_rasp {
        if student_day.date == today {
            return Some(student_day);
        }
    }
    None
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

#[derive(Serialize, Deserialize, Debug)]
struct StudentDay {
    day_of_week: String,
    date: String,
    pairs: Vec<Pair>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Pair {
    number: u8,
    time_period: String,
    name: String,
    pair_type: String,
    teacher: String,
    teams_url_pair: String,
}

fn parse_rasp_html(reps_html: &str) -> Vec<StudentDay> {
    let dom = tl::parse(reps_html, tl::ParserOptions::default()).unwrap();
    let parser = dom.parser();
    let rasp_days = dom.get_elements_by_class_name("day-container");
    let mut result_rasp: Vec<StudentDay> = Vec::new();
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

        if day_of_week == "${dayOfWeek" {
            continue;
        }

        let mut day_pairs: Vec<Pair> = Vec::new();

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

            let pair_tag = pair_info
                .query_selector(parser, ".pair_desc")
                .unwrap()
                .next()
                .unwrap()
                .get(parser)
                .unwrap()
                .as_tag()
                .unwrap();

            let teams_url_pair = pair_tag
                .query_selector(parser, "a[href]")
                .unwrap()
                .next()
                .unwrap()
                .get(parser)
                .unwrap()
                .as_tag()
                .unwrap()
                .attributes()
                .get("href")
                .flatten()
                .unwrap()
                .try_as_utf8_str()
                .unwrap();

            let pair_description = pair_tag.inner_text(parser);

            let pair_desc_splited: Vec<&str> = pair_description.trim().split(",").collect();
            let pair_name = pair_desc_splited[0].trim();
            let pair_type = pair_desc_splited[1].trim();
            let pair_teacher = pair_desc_splited[2].trim();

            let pair = Pair {
                number: pair_number.parse().unwrap(),
                time_period: time_period.to_string(),
                name: pair_name.to_string(),
                pair_type: pair_type.to_string(),
                teacher: pair_teacher.to_string(),
                teams_url_pair: teams_url_pair.to_string(),
            };
            day_pairs.push(pair);
        }

        let student_day = StudentDay {
            day_of_week: day_of_week.to_string(),
            date: date.to_string(),
            pairs: day_pairs,
        };

        result_rasp.push(student_day);
    }
    return result_rasp;
}
