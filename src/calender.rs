use chrono::{Utc, DateTime, TimeZone, NaiveDate, NaiveDateTime, NaiveTime, Datelike};
use chrono::format::Numeric::Timestamp;
use minicaldav::{Calendar, Error, Event};
use url::Url;
use ureq::Agent;
use crate::secrets::{CALENDER_USER, CALENDER_PASS,CALENDER_DAV_LINK};

#[derive(Debug)]
#[derive(Clone)]
pub(crate) struct SimpleEvent {
    pub title: String,
    pub date: NaiveDateTime,
    pub birthday: bool,
}
#[derive(Debug, Clone)]
pub(crate) struct ParseCalenderEventError {
    details: String,
}

pub(crate)  async fn get_calender() -> Result< Vec<SimpleEvent> , ParseCalenderEventError>{
    let agent = Agent::new();
    let url = Url::parse(CALENDER_DAV_LINK).unwrap();
    let mut event_simplified = Vec::<SimpleEvent>::new();

    let calendars = match minicaldav::get_calendars(agent.clone(), CALENDER_USER, CALENDER_PASS, &url) {
        Ok(val) => val,
        Err(e) => return Err(ParseCalenderEventError { details: e.to_string() })
    };
    let calendars = calendars.
        iter().
        filter(|&cal| ["Personal", "Contact birthdays"].contains(&&**cal.name()));
    for calendar in calendars {
        let (mut events, errors) = match minicaldav::get_events(
            agent.clone(),
            CALENDER_USER,
            CALENDER_PASS,
            &calendar) {
            Ok(val) => {val}
            Err(e) => return Err(ParseCalenderEventError { details: e.to_string() })
        };
        events = match calendar.name().as_str() {
            "Personal" => sort_calender_events(events),
            "Contact birthdays" => sort_birthdays(events),
            _ => events
        };

        for event in &events[..2] {
            event_simplified.push(SimpleEvent {
                title: event.get("SUMMARY").unwrap().to_string(),
                date: parse_date_time(&event),
                birthday: matches!(calendar.name().as_str(), "Contact birthdays"),
            })
        }
        if let Some(error) = errors.into_iter().next() {
            println!("Error: {:?}", error);
            return Err(ParseCalenderEventError { details: error.to_string() })
        }
    }
    event_simplified = sort_simplified_events(event_simplified);
    let event_simplified_two= &event_simplified[..2];
    /*for event in event_simplified  {
        println!("{:?}", event);
    }*/
    Ok(event_simplified_two.to_vec())
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
    events.sort_by_key(parse_date_time);

    events.into_iter().
        filter(|ev| parse_date_time(&ev) >= now.naive_utc()).collect()
}

fn sort_birthdays(mut events: Vec<Event>) -> Vec<Event> {
    //let now = NaiveDate::from_ymd_opt(0000, 7, 6).unwrap();
    let now= Utc::now();
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
                ev_date >= now_date
            }).collect()
}
fn sort_simplified_events(mut events: Vec<SimpleEvent>) -> Vec<SimpleEvent>{
    events.sort_by(|ev1,ev2|{
        let ev1_date = NaiveDate::from_ymd_opt(0000, ev1.date.month(), ev1.date.day());
        let ev2_date = NaiveDate::from_ymd_opt(0000, ev2.date.month(), ev2.date.day());
        ev1_date.cmp(&ev2_date)
    });

    events
}