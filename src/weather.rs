use std::fmt;
use serde::Deserialize;
use reqwest;
use image;
use image::{DynamicImage, ImageError};
use reqwest::Error;
use crate::secrets::WEATHER_API_TOKEN;



#[derive(Deserialize, Debug)]
struct OpenWeatherResponse {
    weather: Vec<SkyData>,
    main: WeatherData,
    wind: Wind,
}

#[derive(Deserialize, Debug)]
struct SkyData {
    icon: String,
}

#[derive(Deserialize, Debug)]
struct Wind {
    pub speed: f32,
}

#[derive(Deserialize, Debug)]
struct WeatherData {
    temp: f32,
}

#[derive(Debug)]
pub(crate) struct WeatherResponse {
    pub temp: f32,
    pub wind_speed: f32,
    pub icon_id: String,
    pub icon_img: DynamicImage,
}



static POSITION: (f64, f64) = (49.586200, 11.018097);

#[derive(Debug, Clone)]
pub(crate) struct ParseWeatherError {
    details: String,
}

impl fmt::Display for ParseWeatherError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!(" {} \n Cannot get weather info", self.details))
    }
}

pub(crate) async fn get_weather() -> Result<WeatherResponse, ParseWeatherError> {
    let http_response = match reqwest::get(format!("https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={WEATHER_API_TOKEN}", POSITION.0, POSITION.1)).await {
        Ok(value) => value,
        Err(e) => return Err(ParseWeatherError { details: e.to_string() })
    };
    let response = match http_response.json::<OpenWeatherResponse>().await {
        Ok(value) => value,
        Err(e) => return Err(ParseWeatherError { details: e.to_string() })
    };
    let http_response = match reqwest::get(format!("http://openweathermap.org/img/wn/{}.png", response.weather[0].icon)).await {
        Ok(value) => value,
        Err(e) => return Err(ParseWeatherError { details: e.to_string() })
    };
    let icon_img_bytes = match http_response.bytes().await {
        Ok(value) => value,
        Err(e) => return Err(ParseWeatherError { details: e.to_string() })
    };
    let icon_img = match image::load_from_memory(&icon_img_bytes) {
        Ok(value) => value,
        Err(e) => return Err(ParseWeatherError { details: e.to_string() })
    };


    let weather_response = WeatherResponse {
        temp: response.main.temp - 273.15,
        wind_speed: response.wind.speed,
        icon_id: response.weather[0].icon.clone(),
        icon_img: icon_img,
    };
    //println!("{:#?}", weather_response);
    Ok(weather_response)
}
