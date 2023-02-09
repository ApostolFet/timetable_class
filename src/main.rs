use std::collections::HashMap;



fn main() {
    println!("Hello, world!");
}


fn get_html_timetable() {
    let timetable_url = "https://new.vyatsu.ru/account/obr/rasp/";

    let resp = reqwest::get(timetable_url).expect();
    println!(resp);
}