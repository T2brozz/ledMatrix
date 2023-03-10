use std::{thread, time};
use crate::calender::{get_calender, Simple_Event};
use crate::weather::{get_weather, ParseWeatherError, WeatherResponse};
use rpi_led_panel::{Canvas, RGBMatrix, RGBMatrixConfig};
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


use std::io::Write;
use chrono::{Timelike, Utc, Duration, TimeZone, Offset};
use chrono_tz::Europe::Berlin;
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
        gpio_mapping: String::from("adafruit_hat_pwm"),
        rows: 64,
        cols: 64,
        refresh_rate: 120,
        pi_chip: None,
        pwm_bits: 11,
        pwm_lsb_nanoseconds: 130,
        slowdown: Some(2),
        interlaced: false,
        dither_bits: 0,
        parallel: 1,
        panel_type: None,
        multiplexing: None,
        row_setter: String::from("DirectRowAddressSetter"),
    };
    let rows = config.rows as isize;
    let cols = config.cols as isize;
    let (mut matrix, mut canvas) = RGBMatrix::new(config, 0).expect("Matrix initialization failed");
    let text_style = MonoTextStyle::new(&FONT_8X13, Rgb888::WHITE);
    let red_text_style = MonoTextStyle::new(&FONT_8X13, Rgb888::RED);
    let blue_text_style = MonoTextStyle::new(&FONT_8X13, Rgb888::BLUE);
    let mut last_request_time = Utc::now().timestamp();
    let mut last_response: (WeatherResponse, Vec<Simple_Event>) =
        (get_weather().await.expect("First try to get weather data failed"),
         get_calender().await.expect("First try to get calender events failed")
        );
    let mut current_Event = CurrentEvent { text_scroll: 0.0, event_index: 0 };
    let mut wert = 0.0;
    loop {
        canvas.fill(0, 0, 0);

        let time_now = Utc::now();
        let time_str = time_now.with_timezone(&Berlin).format("%H\n%M\n%S").to_string();
        let clock = Text::new(
            time_str.as_str(),
            Point::new((0) as i32, (8) as i32),
            text_style,
        );
        clock.draw(canvas.as_mut()).unwrap();
        if last_request_time <= time_now.timestamp() - 15 * 60 {
            match get_weather().await {
                Ok(weather) => { last_response.0 = weather }
                Err(_) => {}
            };
            match get_calender().await {
                Ok(events) => last_response.1 = events,
                Err(_) => {}
            };
            last_request_time = time_now.timestamp();
            println!("wuu es geht");
        }
        let temperature_string = format!("{:.1}C", last_response.0.temp);
        let temperature = Text::new(
            temperature_string.as_str(),
            Point::new((20) as i32, (8) as i32),
            red_text_style,
        );
        temperature.draw(canvas.as_mut()).unwrap();
        let newiamge = last_response.0.icon_img.thumbnail(26, 26);
        let image_data = ImageRawBE::<Rgb888>::new(newiamge.as_bytes(), wert as u32);
        let image = Image::new(
            &image_data,
            Point::new(10, 10),
        );
        //image.draw(canvas.as_mut()).unwrap();

        let calenderevent = Text::new(
            &last_response.1[current_Event.event_index].title,
            Point::new((current_Event.text_scroll) as i32, (45) as i32),
            blue_text_style,
        );
        calenderevent.draw(canvas.as_mut()).unwrap();

        canvas = matrix.update_on_vsync(canvas);
        current_Event.text_scroll-=0.07;
        if current_Event.text_scroll < -20.0 {
            current_Event.text_scroll=0.0;
        }
    }
}
