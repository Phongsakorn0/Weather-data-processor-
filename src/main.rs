use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use tokio::sync::Mutex;
use std::sync::Arc;

const FOLDER_PATH: &str = "/Users/user/Desktop/test"; // Change to your desired folder path

#[derive(Serialize, Deserialize, Debug)]
struct WeatherData {
    pressure: f64,
    relative_humidity: f64,
    temperature: f64,
    wind_direction: f64,
    wind_speed: f64,
    chp1: f64,
    direct_sun: f64,
    global_sun: f64,
    diffuse_sun: f64,
    rain_fall: f64,
    all_day_illumination: f64,
    pm25: f64,
    timestamp: String,
}

struct FileTracker {
    last_file_name: Option<String>,
    last_line_index: usize,
}

impl FileTracker {
    fn new() -> Self {
        Self {
            last_file_name: None,
            last_line_index: 0,
        }
    }

    fn reset(&mut self, file_name: &str) {
        self.last_file_name = Some(file_name.to_string());
        self.last_line_index = 0;
    }

    fn update_last_line(&mut self, line_index: usize) {
        self.last_line_index = line_index;
    }
}

async fn read_and_send_data(
    client: &reqwest::Client,
    file_name: &str,
    file_tracker: Arc<Mutex<FileTracker>>,
) {
    let file_path = Path::new(FOLDER_PATH).join(file_name);
    println!("File path: {:?}", file_path);

    if !file_path.exists() {
        eprintln!("File not found: {:?}", file_path);
        return;
    }

    let bytes = match tokio::fs::read(&file_path).await {
        Ok(bytes) => bytes,
        Err(err) => {
            eprintln!("Failed to read file {:?}: {}", file_path, err);
            return;
        }
    };

    let content = String::from_utf8_lossy(&bytes);
    let lines: Vec<&str> = content.lines().collect();
    let mut data: Vec<WeatherData> = Vec::new();

    // อ่านข้อมูลจากตัวเก็บค่า (Tracker)
    let mut tracker = file_tracker.lock().await;
    let start_line = if let Some(last_file_name) = &tracker.last_file_name {
        if last_file_name == file_name {
            tracker.last_line_index + 1 // เริ่มจากแถวถัดไป
        } else {
            tracker.reset(file_name); // ไฟล์ใหม่, รีเซ็ตค่า
            17 // เริ่มที่แถวที่ 17
        }
    } else {
        tracker.reset(file_name);
        17
    };

    println!("Start processing from line {}", start_line);

    for row_index in start_line..2000 {
        if let Some(row_content) = lines.get(row_index) {
            let row_content = row_content.trim();
            if row_content.is_empty() {
                println!("Skipping empty line at row {}", row_index);
                continue;
            }
            // println!("Processing line {}: {}", row_index, row_content);

            let columns: Vec<&str> = row_content.split(',').collect();

            let weather_data = WeatherData {
                pressure: columns[16].parse::<f64>().unwrap_or(0.0),
                relative_humidity: columns[17].parse::<f64>().unwrap_or(0.0),
                temperature: columns[18].parse::<f64>().unwrap_or(0.0),
                wind_direction: columns[19].parse::<f64>().unwrap_or(0.0),
                wind_speed: columns[20].parse::<f64>().unwrap_or(0.0),
                chp1: columns[21].parse::<f64>().unwrap_or(0.0),
                direct_sun: columns[22].parse::<f64>().unwrap_or(0.0),
                global_sun: columns[23].parse::<f64>().unwrap_or(0.0),
                diffuse_sun: columns[24].parse::<f64>().unwrap_or(0.0),
                rain_fall: columns[25].parse::<f64>().unwrap_or(0.0),
                all_day_illumination: columns[26].parse::<f64>().unwrap_or(0.0),
                pm25: columns[27].parse::<f64>().unwrap_or(0.0),
                timestamp: Utc::now().to_rfc3339(),
            };

            data.push(weather_data);
            tracker.update_last_line(row_index); // อัปเดตแถวล่าสุดที่อ่าน
        } else {
            println!("Row {} is out of bounds. Stopping processing.", row_index);
            break;
        }
    }

    drop(tracker); // ปลดล็อก Mutex

    if data.is_empty() {
        println!("No new data found in the file. Nothing to send.");
        return;
    }

    for row in &data {
        println!("{:?}", row);
    }

    for row in &data {
        match client.post("http://localhost:8080/weather").json(row).send().await {
            Ok(response) => println!("Data posted successfully: {:?}", response),
            Err(err) => eprintln!("Error posting data: {}", err),
        }
    }
    
}

// async fn post_data_to_api(client: &Client, data: &[WeatherData]) {
//     for row in data {
//         match client
//             .post("http://localhost:8080/weather")
//             .json(row)
//             .send()
//             .await
//         {
//             Ok(response) => println!("Data posted successfully: {:?}", response),
//             Err(err) => eprintln!("Error posting data: {}", err),
//         }
//     }
// }

// async fn get_weather(client: &Client) {
//     match client.get("http://localhost:8080/weather").send().await {
//         Ok(response) => match response.json::<serde_json::Value>().await {
//             Ok(weather) => println!("Weather Data: {:?}", weather),
//             Err(err) => eprintln!("Error parsing weather data: {}", err),
//         },
//         Err(err) => eprintln!("Error fetching weather data: {}", err),
//     }
// }

// fn get_file_name_in_folder() -> String {
//     let mut file_names = String::new();
//     if let Ok(pond) = fs::read_dir(&FOLDER_PATH) {
//         for entry in pond {
//             if let Ok(entry) = entry {
//                 // Here, entry is a DirEntry.
//                 file_names.push_str(&format!("{:?}\n", entry.file_name()));
//             }
//         }
//     }
//     return file_names;
// }

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

    let file_tracker = Arc::new(Mutex::new(FileTracker::new()));

    // Create an interval that ticks every 1 minute
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

    loop {
        // Wait for the next tick
        interval.tick().await;

        // Get the most recent file
        let recent_file = get_recent_file_name_in_folder();

        // Process the file
        if !recent_file.is_empty() {
            println!("Processing file: {}", recent_file);
            read_and_send_data(&client, &recent_file, file_tracker.clone()).await;
        } else {
            println!("No recent file found in the folder.");
        }
    }
}
