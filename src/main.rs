use std::{thread, time};
use crate::calender::get_calender;
use crate::weather::{get_weather, ParseWeatherError, WeatherResponse};
use rpi_led_panel::{Canvas, RGBMatrix, RGBMatrixConfig};
use tokio::time::{sleep};
use embedded_graphics::{
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
use chrono::{Timelike, Utc,Duration};
use image::codecs::png::CompressionType::Default;
use serde::de::Unexpected::Option;

fn scale_col(value: isize, low: isize, high: isize) -> u8 {
    if value < low {
        return 0;
    }
    if value > high {
        return 255;
    }
    (255 * (value - low) / (high - low)) as u8
}

fn rotate([x, y]: [isize; 2], angle: f64) -> [f64; 2] {
    [
        x as f64 * angle.cos() - y as f64 * angle.sin(),
        x as f64 * angle.sin() + y as f64 * angle.cos(),
    ]
}
#[tokio::main]
async fn main() {
    let config: RGBMatrixConfig= RGBMatrixConfig{
        gpio_mapping:String::from("adafruit_hat_pwm"),
        rows: 64,
        cols: 64,
        refresh_rate:  120 ,
        pi_chip:  None,
        pwm_bits: 11 ,
        pwm_lsb_nanoseconds: 130 ,
        slowdown: Some(2) ,
        interlaced: false ,
        dither_bits: 0 ,
        parallel: 1 ,
        panel_type: None ,
        multiplexing: None,
        row_setter:   String::from("DirectRowAddressSetter")
    };
    let rows = config.rows as isize;
    let cols = config.cols as isize;
    let (mut matrix, mut canvas) = RGBMatrix::new(config, 0).expect("Matrix initialization failed");
    let text_style=MonoTextStyle::new(&FONT_8X13, Rgb888::WHITE);
    let mut last_request_time=Utc::now().timestamp();
    let mut last_response:(WeatherResponse)=get_weather().await.expect("First try to get weather data failed");
    loop{
        canvas.fill(0, 0, 0);

        let time_now = Utc::now();
        let time_str=time_now.format("%H\n%M\n%S").to_string();
        let clock = Text::new(
            time_str.as_str(),
            Point::new((0) as i32, (8) as i32),
            text_style
        );
        clock.draw(canvas.as_mut()).unwrap();
        if last_request_time <=  time_now.timestamp()-15*60 {
            match get_weather().await {
                Ok(w) => {last_response=w}
                Err(_) => {}
            }
            last_request_time=Utc::now().timestamp();
            println!("wuu es geht");
        }
        let temperature_string =format!("{:.1}°C", last_response.temp);
        let temperature = Text::new(
            temperature_string.as_str(),
            Point::new((20) as i32, (8) as i32),
            text_style
        );
        temperature.draw(canvas.as_mut()).unwrap();


        canvas = matrix.update_on_vsync(canvas);



    }

}
