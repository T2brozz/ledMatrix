use std::{thread, time};
use crate::calender::{get_calender, SimpleEvent};
use crate::weather::{get_weather, ParseWeatherError, WeatherResponse};
use rpi_led_panel::{Canvas, HardwareMapping, LedSequence, RGBMatrix, RGBMatrixConfig, RowAddressSetterType};
use tokio::time::{sleep};
use embedded_graphics::{
    image::{Image, ImageRawBE},
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle},
    text::{Alignment, Text},
    Drawable,
};

mod weather;
mod calender;
mod secrets;


use chrono::{Timelike, Utc, Duration, TimeZone, Offset, NaiveTime, DateTime};

use chrono_tz::Europe::Berlin;
use embedded_graphics::image::{ImageRaw, ImageRawLE};
use image::codecs::png::CompressionType::Default;
use image::EncodableLayout;
use image::imageops::FilterType;
use serde::de::Unexpected::Option;

struct CurrentEvent {
    text_scroll: f32,
    event_index: usize,
}

#[tokio::main]
async fn main() {
    let config: RGBMatrixConfig = RGBMatrixConfig {
        hardware_mapping: HardwareMapping::adafruit_hat_pwm(),
        rows: 64,
        cols: 64,
        refresh_rate: 120,
        pi_chip: None,
        pwm_bits: 11,
        pwm_lsb_nanoseconds: 130,
        slowdown: Some(2),
        interlaced: false,
        dither_bits: 0,
        chain_length: 1,
        parallel: 1,
        panel_type: None,
        multiplexing: None,
        row_setter: RowAddressSetterType::Direct,
        led_sequence: LedSequence::Gbr,
    };
    let (mut matrix, mut canvas) = RGBMatrix::new(config, 0).expect("Matrix initialization failed");
    let text_style = MonoTextStyle::new(&FONT_8X13, Rgb888::WHITE);
    let red_text_style = MonoTextStyle::new(&FONT_8X13, Rgb888::RED);
    let blue_text_style = MonoTextStyle::new(&FONT_8X13, Rgb888::BLUE);
    let mut last_request_time = Utc::now().timestamp();
    let mut last_response: (WeatherResponse, Vec<SimpleEvent>) =
        (get_weather().await.expect("First try to get weather data failed"),
         get_calender().await.expect("First try to get calender events failed")
        );
    let mut current_event = CurrentEvent { text_scroll: 5.0, event_index: 0 };

    let new_image = last_response.0.icon_img;
    println!("{:?}", new_image.width());
    //println!("{:?}", new_image.as_bytes());
    new_image.save("a.jpg");
    let image_data = ImageRaw::<Rgb888>::new(new_image.as_bytes(), new_image.width());
    let image = Image::new(
        &image_data,
        Point::new(20_i32, 10_i32),
    );
    loop {
        let time_now = Utc::now();

        canvas.fill(0, 0, 0);


        let time_str = time_now.with_timezone(&Berlin).format("%H\n%M\n%S").to_string();
        let clock = Text::new(
            time_str.as_str(),
            Point::new(0_i32, 8_i32),
            text_style,
        );
        clock.draw(canvas.as_mut()).unwrap();

        let date_str = time_now.with_timezone(&Berlin).format("%d.%m").to_string();
        let date = Text::new(
            date_str.as_str(),
            Point::new(20_i32, 10_i32),
            text_style,
        );
        date.draw(canvas.as_mut()).unwrap();

        if last_request_time <= time_now.timestamp() - 15 * 60 {
            if let Ok(weather) = get_weather().await { last_response.0 = weather };
            if let Ok(events) = get_calender().await { last_response.1 = events };
            last_request_time = time_now.timestamp();
            println!("wuu es geht");
        }
        let temperature_string = format!("{:.1}C", last_response.0.temp);
        let temperature = Text::new(
            temperature_string.as_str(),
            Point::new(20_i32, 25_i32),
            red_text_style,
        );
        temperature.draw(canvas.as_mut()).unwrap();


        let calender_event = Text::new(
            &last_response.1[current_event.event_index].title,
            Point::new((current_event.text_scroll) as i32, 45_i32),
            if last_response.1[current_event.event_index].birthday {
                red_text_style
            } else {
                blue_text_style
            },
        );
        calender_event.draw(canvas.as_mut()).unwrap();

        let date_string = last_response.1[current_event.event_index].date.format("%d.%m").to_string();
        let calender_date = Text::new(
            date_string.as_str(),
            Point::new(12, 60_i32),
            if last_response.1[current_event.event_index].birthday {
                red_text_style
            } else {
                blue_text_style
            },
        );
        calender_date.draw(canvas.as_mut()).unwrap();

        canvas = matrix.update_on_vsync(canvas);

        current_event.text_scroll -= 0.2;
        if current_event.text_scroll < -(last_response.1[current_event.event_index].title.chars().count() as i32 * 8) as f32 {
            current_event.text_scroll = 5.0;
            current_event.event_index += 1;
            if current_event.event_index >= last_response.1.len() {
                current_event.event_index = 0;
            }
        }
    }
}
