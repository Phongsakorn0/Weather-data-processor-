use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::{fs, path::Path};

const FOLDER_PATH: &str = "/Users/user/Desktop/test"; // Change to your desired folder path
const FILE_PREFIX: &str = "weather_data_";

#[derive(Serialize, Deserialize, Debug)]
struct WeatherData {
    pressure: f64,
    relative_humidity: u32,
    temperature: f64,
    wind_direction: u32,
    wind_speed: u32,
    chp1: f64,
    direct_sun: u32,
    global_sun: u32,
    diffuse_sun: u32,
    rain_fall: f64,
    all_day_illumination: u32,
    pm25: u32,
    timestamp: String,
}

async fn read_and_send_data(client: &Client, file_name: &str) {
    let file_path = Path::new(FOLDER_PATH).join(file_name);
    println!("File path: {:?}", file_path);

    match fs::read(&file_path) {
        Ok(bytes) => {
            println!("Reading file: {}", file_name);

            // Interpret the file content as UTF-8
            let content = String::from_utf8_lossy(&bytes);

            // Split content into lines
            let lines: Vec<&str> = content.lines().collect();

            // Simulate reading data from the file
            let mut data: Vec<WeatherData> = Vec::new();

            // Loop through rows 18 to 44 (adjusted index: 17 to 43)
            for row_index in 17..30 {
                let row_content = lines.get(row_index).unwrap_or(&"");
                let mut row_data = HashMap::new();
                let mut has_data = false;

                for (col_index, col) in ["Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "AA", "AB"]
                    .iter()
                    .enumerate()
                {
                    // Use each line content as the simulated value
                    let simulated_value = row_content.trim(); // Line content as value
                    if !simulated_value.is_empty() {
                        has_data = true;
                        row_data.insert(format!("{}{}", col, row_index + 1), simulated_value);
                        println!("{}{}: {}", col, row_index + 1, row_data[&format!("{}{}", col, row_index + 1)]);  
                    }
                }

                if has_data {
                    let weather_data = WeatherData {
                        pressure: row_data["Q"].parse::<f64>().unwrap_or(0.0),
                        relative_humidity: row_data["R"].parse::<u32>().unwrap_or(0),
                        temperature: row_data["S"].parse::<f64>().unwrap_or(0.0),
                        wind_direction: row_data["T"].parse::<u32>().unwrap_or(0),
                        wind_speed: row_data["U"].parse::<u32>().unwrap_or(0),
                        chp1: row_data["V"].parse::<f64>().unwrap_or(0.0),
                        direct_sun: row_data["W"].parse::<u32>().unwrap_or(0),
                        global_sun: row_data["X"].parse::<u32>().unwrap_or(0),
                        diffuse_sun: row_data["Y"].parse::<u32>().unwrap_or(0),
                        rain_fall: row_data["Z"].parse::<f64>().unwrap_or(0.0),
                        all_day_illumination: row_data["AA"].parse::<u32>().unwrap_or(0),
                        pm25: row_data["AB"].parse::<u32>().unwrap_or(0),
                        timestamp: Utc::now().to_rfc3339(),
                    };
                    data.push(weather_data);
                } else {
                    println!("No data found in line: {}. Stopping further processing.", row_content);
                    break; // Stop processing if no data is found in the current line
                }
            }

            if data.is_empty() {
                println!("No valid data found in the file. Nothing to send.");
                return;
            }

            for row in &data {
                println!("{:?}", row);
            }

            // Uncomment to send data after preparing
            /*
            for row in &data {
                match client
                    .post("http://localhost:8080/weather")
                    .json(row)
                    .send()
                    .await
                {
                    Ok(response) => println!("Data posted successfully: {:?}", response),
                    Err(err) => eprintln!("Error posting data: {}", err),
                }
            }
            */
        }
        Err(err) => {
            eprintln!("Failed to read file {:?}: {}", file_path, err);
        }
    }
}
async fn post_data_to_api(client: &Client, data: &[WeatherData]) {
    for row in data {
        match client
            .post("http://localhost:8080/weather")
            .json(row)
            .send()
            .await
        {
            Ok(response) => println!("Data posted successfully: {:?}", response),
            Err(err) => eprintln!("Error posting data: {}", err),
        }
    }
}

async fn get_weather(client: &Client) {
    match client.get("http://localhost:8080/weather").send().await {
        Ok(response) => match response.json::<serde_json::Value>().await {
            Ok(weather) => println!("Weather Data: {:?}", weather),
            Err(err) => eprintln!("Error parsing weather data: {}", err),
        },
        Err(err) => eprintln!("Error fetching weather data: {}", err),
    }
}

fn get_file_name_in_folder() -> String {
    let mut file_names = String::new();
    if let Ok(pond) = fs::read_dir(&FOLDER_PATH) {
        for entry in pond {
            if let Ok(entry) = entry {
                // Here, entry is a DirEntry.
                file_names.push_str(&format!("{:?}\n", entry.file_name()));
            }
        }
    }
    return file_names;
}

fn get_recent_file_name_in_folder() -> String {
    let file_names = String::new();
    let mut recent_file = String::new();
    let mut recent_time = u64::MAX;
    if let Ok(entries) = fs::read_dir(&FOLDER_PATH) {
        for entry in entries {
            if let Ok(entry) = entry {
                // Here, entry is a DirEntry.
                let file_name = entry.file_name();
                let file_name_str = file_name.to_string_lossy();
                let metadata = fs::metadata(&entry.path()).unwrap();
                let modified_time = metadata.modified().unwrap().elapsed().unwrap().as_secs();
                if modified_time < recent_time {
                    recent_time = modified_time;
                    recent_file = file_name_str.to_string();
                }
            }
        }
    }
    return recent_file;
}

#[tokio::main]
async fn main() {
    let client = Client::new();

    let _recent_file = get_recent_file_name_in_folder();

    read_and_send_data(&client, &_recent_file).await;
    // let content = read_file(_recent_file);
    // print!("{:?}", content);

    // GET and POST example

    // let example_weather = WeatherData {
    //     pressure: 1013.25,
    //     relative_humidity: 85,
    //     temperature: 30.5,
    //     wind_direction: 90,
    //     wind_speed: 5,
    //     chp1: 1.2,
    //     direct_sun: 400,
    //     global_sun: 800,
    //     diffuse_sun: 200,
    //     rain_fall: 10.5,
    //     all_day_illumination: 1200,
    //     pm25: 15,
    //     timestamp: Utc::now().to_rfc3339(),
    // };

    // post_data_to_api(&client, &[example_weather]).await;

    // get_weather(&client).await;

    // // Schedule the task to run every minute
    // let scheduler = JobScheduler::new().await.unwrap();
    // scheduler
    //     .add(
    //         Job::new_async("* * * * * *", move |_uuid, _l| {
    //             let client_clone = client.clone();
    //             Box::pin(async move {
    //                 read_and_send_data(&client_clone).await;
    //             })
    //         })
    //         .unwrap(),
    //     )
    //     .unwrap();

    // scheduler.start().await.unwrap();
}
