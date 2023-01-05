//use std::{thread, time};
use crate::calender::get_calender;
use crate::weather::{get_weather, ParseWeatherError};
use rpi_led_matrix::{LedMatrix, LedColor,LedMatrixOptions};
use tokio::time::{sleep,Duration};

mod weather;
mod calender;
mod secrets;

#[tokio::main]
async fn main()-> Result<(),()>
{
    /*
        let weather =match get_weather().await{
            Ok(value)=> value,
            Err(e)=> return Err(())
        };
        println!("{:#?}",weather);
    */

    //get_calender();


    let mut options = LedMatrixOptions::new();
    options.set_hardware_mapping("adafruit-hat-pwm");
    options.set_cols(64);
    options.set_rows(64);
    //options.set_hardware_pulsing(false);
    options.set_led_rgb_sequence("BRG");

    let matrix = LedMatrix::new(Some(options), None).unwrap();
    let mut canvas = matrix.offscreen_canvas();
    /*for red in 0..255 {
        for green in 0..255 {
            for blue in 0..255 {
                canvas.fill(&LedColor { red, green, blue });
                canvas = matrix.swap(canvas);
                sleep(Duration::from_millis(10)).await;
            }
        }
    }*/
    loop {
        canvas.fill(&LedColor { red: 127, green: 0, blue: 0 });
        canvas = matrix.swap(canvas);
    }
    Ok(())
}
