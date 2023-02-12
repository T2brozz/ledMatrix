use chrono::{Utc, DateTime, TimeZone, NaiveDate, NaiveDateTime, NaiveTime, Datelike};
use chrono::format::Numeric::Timestamp;
use minicaldav::{Calendar, Event};
use url::Url;
use ureq::Agent;
use crate::secrets::{CALENDER_USER, CALENDER_PASS};

#[derive(Debug)]
struct Simple_Event {
    title: String,
    date: NaiveDateTime,
    birthday: bool,
}

pub fn get_calender() {
    let agent = Agent::new();
    let url = Url::parse("https://dav.mailbox.org").unwrap();
    let mut event_simplified = Vec::<Simple_Event>::new();

    let calendars = minicaldav::get_calendars(agent.clone(), CALENDER_USER, CALENDER_PASS, &url).unwrap();
    let calendars = calendars.
        iter().
        filter(|&cal| ["Calendar", "Birthdays"].contains(&&**cal.name()));
    for calendar in calendars {
        println!("{:?}", calendar);
        let (mut events, errors) = minicaldav::get_events(
            agent.clone(),
            CALENDER_USER,
            CALENDER_PASS,
            &calendar).
            unwrap();

        events = match calendar.name().as_str() {
            "Calendar" => sort_calender_events(events),
            "Birthdays" => sort_birthdays(events),
            _ => events
        };

        for event in &events[..2] {
            event_simplified.push(Simple_Event {
                title: event.get("SUMMARY").unwrap().to_string(),
                date: parse_date_time(&event),
                birthday: match calendar.name().as_str() {
                    "Birthdays" => true,
                    _ => false
                },
            })
        }
        for error in errors {
            println!("Error: {:?}", error);
        }
    }
    event_simplified = sort_simplified_events(event_simplified);
    for event in event_simplified  {
        println!("{:?}", event);
    }
}

fn parse_date_time(event: &Event) -> NaiveDateTime {
    let time_str = event.get("DTSTART").unwrap();

    match NaiveDateTime::parse_from_str(time_str, "%Y%m%dT%H%M%S%Z") {
        Ok(value) => value,
        Err(e) => match NaiveDateTime::parse_from_str(time_str, "%Y%m%dT%H%M%S") {
            Ok(value) => value,
            Err(e) => NaiveDateTime::new(NaiveDate::parse_from_str(time_str, "%Y%m%d").expect("Third Time convert failed"), NaiveTime::default()),
        },
    }
}

fn sort_calender_events(mut events: Vec<Event>) -> Vec<Event> {
    let now = Utc::now();
    events.sort_by(|ev1, ev2|
        parse_date_time(ev1).cmp(&parse_date_time(ev2)));

    events.into_iter().
        filter(|ev| parse_date_time(&ev) >= now.naive_utc()).collect()
}

fn sort_birthdays(mut events: Vec<Event>) -> Vec<Event> {
    let now = NaiveDate::from_ymd_opt(0000, 7, 6).unwrap();
    //let now= Utc::now(); //TODO
    events.sort_by(|ev1, ev2|
        {
            let ev1_date = parse_date_time(ev1);
            let ev2_date = parse_date_time(ev2);

            let ev1_date = NaiveDate::from_ymd_opt(0000, ev1_date.month(), ev1_date.day());
            let ev2_date = NaiveDate::from_ymd_opt(0000, ev2_date.month(), ev2_date.day());
            ev1_date.cmp(&ev2_date)
        }
    );
    events.into_iter().
        filter(|ev|
            {
                let ev_date = parse_date_time(ev);
                let ev_date = NaiveDate::from_ymd_opt(0000, ev_date.month(), ev_date.day()).unwrap();
                let now_date = NaiveDate::from_ymd_opt(0000, now.month(), now.day()).unwrap();
                return ev_date >= now_date;
            }
        ).collect()
}
fn sort_simplified_events(mut events: Vec<Simple_Event>) -> Vec<Simple_Event>{
    events.sort_by(|ev1,ev2|{
        let ev1_date = NaiveDate::from_ymd_opt(0000, ev1.date.month(), ev1.date.day());
        let ev2_date = NaiveDate::from_ymd_opt(0000, ev2.date.month(), ev2.date.day());
        ev1_date.cmp(&ev2_date)
    });

    return events;
}