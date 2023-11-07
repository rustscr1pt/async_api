use scraper::{Html, Selector};
use urlencoding::encode;
use crate::data_structs::{GroupEvent, Themes, MapsToMatch, SubwayColors};

pub async fn get_request() -> String {
    return reqwest::get("https://na-msk.ru/schedule-member/").await.unwrap().text().await.unwrap();
}

pub async fn na_collected() -> Vec<GroupEvent>  {
    println!("Started collecting the Vec of all groups.");
    let mut link_vec : Vec<String> = Vec::new();
    let request = get_request().await;
    let document = Html::parse_document(&request);
    let time_length = Selector::parse("p.mob-none").unwrap();
    let city_selector = Selector::parse("p.new-sked-group-adress.fs-16.fs-md-14.fs-xl-16").unwrap();
    let selector_name = Selector::parse("p.new-sked-group-name.fs-20.fs-md-16.fs-xl-20.fw-700>a").unwrap();
    let selector_adress = Selector::parse("p.new-sked-group-adress.fs-16.fs-md-14.fs-xl-16.mob-none").unwrap();
    let time_selector = Selector::parse("div.new-sked-group-time-block>span").unwrap();
    let link_selector = Selector::parse("p.new-sked-group-name.fs-20.fs-md-16.fs-xl-20.fw-700>a").unwrap();
    let theme_selector = Selector::parse("p.new-sked-description-text.fs-8.fs-md-14.fs-xl-16").unwrap();

    for elements in document.select(&link_selector) {
        if let Some(href) = elements.value().attr("href") {
            let formatted = format!("https://na-msk.ru/groups/{}?previous_path=schedule-member", format_rus(&href));
            println!("{}", formatted);
            link_vec.push(formatted)
        }
    }
    let ready_name : Vec<String> = document
        .select(&selector_name)
        .map(|name| name.inner_html())
        .collect();
    let ready_adress : Vec<String> = document
        .select(&selector_adress)
        .map(|adress| adress.inner_html())
        .collect();
    let my_time : Vec<String> = document
        .select(&time_selector)
        .map(|time| time.inner_html())
        .collect();
    let length : Vec<String> = document
        .select(&time_length)
        .map(|time| time.inner_html())
        .collect();
    let ready_city : Vec<String> = document
        .select(&city_selector)
        .map(|city| city.inner_html())
        .collect();

    let themes_collected = get_thematics(document, theme_selector);

    let formatted_time = sort_schedule(length);
    let answer = manage_vec(my_time, ready_name, ready_adress, formatted_time, link_vec, parse_cities(ready_city), themes_collected);
    println!("{:?}", answer);
    println!("Collected a vec of GroupEvent");
    return answer
}

fn parse_cities(ready_city : Vec<String>) -> Vec<String> {
    let mut collected_cities = Vec::new();
    for elements in ready_city {
        let splited : Vec<&str> = elements.trim().split(",").collect();
        println!("{:?}", splited);
        let chars : Vec<char> = splited
            .get(0)
            .expect("Couldn't extract the city")
            .chars()
            .collect();
        if chars[0] == 'г' && chars[1] == '.' && chars[2] == ' ' {
            let mut add : bool = true;
            for slices in splited.iter() {
                for chars in slices.chars() {
                    if chars.is_numeric() {
                        add = false
                    }
                    if chars == '\n' {
                        break
                    }
                }
            }
            if add {
                collected_cities.push(splited.get(0).expect("Couldn't get an element.").to_string())
            }
        }
    }
    println!("{:?}", collected_cities);
    return collected_cities
}

fn format_rus(href : &str) -> String {
    let first_vec : Vec<&str> = href.split("/").collect();
    let second_split : Vec<&str> = first_vec
        .get(2)
        .expect("Couldn't get an item by index while formatting.")
        .split("?")
        .collect();
    let formatted = encode(second_split
        .get(0)
        .expect("Couldn't get a name of item."));
    return formatted.to_string()
}

fn sort_schedule(length : Vec<String>) -> Vec<String> {
    let mut formatted_length : Vec<String> = Vec::new();
    for (count, elements) in length.iter().enumerate() {
        if count % 3 == 0 {
            formatted_length.push(elements.to_string());
        }
        else {
            println!("Element is dropped.")
        }
    }
    println!("{:?}", formatted_length);
    return formatted_length
}

fn manage_vec(array : Vec<String>,
              name : Vec<String>,
              placement : Vec<String>,
              schedule : Vec<String>,
              links : Vec<String>,
              city : Vec<String>, thematics : Vec<Themes>) -> Vec<GroupEvent> {
    let mut return_vec : Vec<GroupEvent> = Vec::new();
    let mut sorted_vec : Vec<String> = Vec::new();
    println!("{:?}", array);
    let mut new_string : String = String::new();
    for chars in &array {
        if new_string.len() == 2 {
            new_string.push(':')
        }
        new_string.push_str(&chars);
        if new_string.len() == 5 {
            sorted_vec.push(new_string);
            new_string = String::new()
        }
    }
    println!("{:?}", sorted_vec);
    for (index, object) in sorted_vec.iter().enumerate() {
        println!("\nGroup name : {1}\nGroup's place : {2}\nTime : {0}", object, name[index].trim(), placement[index].trim());
        return_vec.push(GroupEvent {
            group_name : name[index].trim().to_string(),
            place : placement[index].trim().to_string(),
            time : object.to_string(),
            schedule : schedule[index].trim().to_string(),
            link : links[index].trim().to_string(),
            city : city[index].trim().to_string(),
            thematics : thematics[index].clone(),
            yandex_maps : "".to_owned(),
            subway_colored : Vec::new(),
        })
    }
    return return_vec
}

fn get_thematics(html : Html, theme_selector : Selector) -> Vec<Themes> {
    let children = html
        .select(&theme_selector)
        .flat_map(|value| value.text())
        .collect::<Vec<&str>>();
    let mut reformed : Vec<Themes> = Vec::with_capacity(&children.len() / 3);
    for (index, tracks) in children.iter().enumerate() {
        if tracks.trim() == "Темы" {
            let mut appendable : Vec<String> = Vec::new();
            for numbers in index..&children.len()-1 {
                if children[numbers + 1].trim() != "" {
                    appendable.push(children[numbers + 1].trim().to_string())
                }
                else {
                    break
                }
            }
            reformed.push(Themes { theme : closed_meeting_filler(appendable) }) // Added a filter for a closed meeting in case there is empty field.
        }
    }
    return reformed
}

fn closed_meeting_filler(appendable : Vec<String>) -> Vec<String> { // filter for closed meetings.
    let mut returnable : Vec<String> = Vec::new();
    if appendable.len() == 0 {
        returnable.push("Закрытое собрание".to_string());
        return returnable
    }
    return appendable
}

pub async fn retrieve_maps(linked : String) -> MapsToMatch {
    let mut collected : Vec<String> = Vec::new();
    let request = reqwest::get(&linked).await.unwrap().text().await.unwrap();
    let document = Html::parse_document(&request);
    let map_selector = Selector::parse("iframe").unwrap();
    for elements in document.select(&map_selector) {
        if let Some(link) = elements.value().attr("src") {
            collected.push(link.to_string())
        }
    }
    println!("Retrieved yandex.maps widget for {}", linked);
    let get_text = Selector::parse("p.member-service-mko-text-medium.fw-500.fs-14.fs-md-16.fs-xl-20 > span").unwrap();
    let mut presenter : Vec<SubwayColors> = Vec::new();
    let subway_names = document
        .select(&get_text)
        .flat_map(|value| value.text())
        .collect::<Vec<&str>>()
        .iter()
        .map(|element| element.to_string())
        .collect::<Vec<String>>();
    let colors_unfiltered = document
        .select(&get_text)
        .map(|value| value.value().attr("style").expect("No style value.").split(" ").collect::<Vec<&str>>())
        .collect::<Vec<Vec<&str>>>()
        .iter()
        .map(|value| value.get(1).expect("Couldn't get a color").to_string())
        .collect::<Vec<String>>();
    println!("Collected VECS with colors_unfiltered & subway_names for {}", linked);
    for (index, elements) in subway_names.iter().enumerate() {
        presenter.push(SubwayColors {
            subway : elements.to_string(),
            color : filter_exceptions(index, elements.to_string(), colors_unfiltered.to_owned())
        })
    }
    println!("Ready to return MapsToMatch object {}", linked);
    return MapsToMatch {
        link : linked,
        map : filter_out(collected.get(0).expect("Couldn't get a value in maps.").to_string()),
        subway : presenter
    }
}

fn filter_out(filtration : String) -> String {
    for element in filtration.chars() {
        if element == '<' {
            return "Null".to_string()
        }
    }
    return filtration
}

fn filter_exceptions(index : usize, elements : String, colors_unfiltered : Vec<String>) -> String {
    if elements == "Верхние Лихоборы".to_string() {
        return "#BED12C".to_string()
    }
    if elements == "Петровский парк".to_string() {
        return "#30d5c8".to_string()
    }
    if elements == "Новопеределкино".to_string() {
        return "#FFCD1C".to_string()
    }
    return colors_unfiltered[index].to_string()
}
