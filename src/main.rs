use actix_files as fs;
use actix_files::NamedFile;
use actix_web::{web, App, HttpServer, HttpResponse, Responder, post, get, Error};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct ModbusData {
    device: String,
    address: u16,
    count: Option<u16>,
    values: Option<Vec<u16>>,
}

struct AppState {
    http_client: Client,
}

#[post("/read")]
async fn read_registers(data: web::Json<ModbusData>, state: web::Data<AppState>) -> impl Responder {
    let client = &state.http_client;
    let response = client
        .get("http://127.0.0.1:5000/read")
        .query(&[
            ("device", data.device.as_str()),
            ("address", &data.address.to_string()),
            ("count", &data.count.unwrap_or(2).to_string()),
        ])
        .send()
        .await
        .expect("Failed to send request");
    
    let data: serde_json::Value = response.json().await.expect("Failed to parse JSON");

    HttpResponse::Ok().json(data)
}

#[post("/write")]
async fn write_registers(data: web::Json<ModbusData>, state: web::Data<AppState>) -> impl Responder {
    let client = &state.http_client;
    let response = client
        .post("http://127.0.0.1:5000/write")
        .json(&*data)
        .send()
        .await
        .expect("Failed to send request");
    
    let result: serde_json::Value = response.json().await.expect("Failed to parse JSON");

    HttpResponse::Ok().json(result)
}

#[get("/")]
async fn index() -> Result<NamedFile, Error> {
    NamedFile::open("static/index.html").map_err(|_| actix_web::error::ErrorNotFound("File not found"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = Client::new();
    let app_state = web::Data::new(AppState {
        http_client: client,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(read_registers)
            .service(write_registers)
            .service(index)
            // Serve all static files from the static folder
            .service(fs::Files::new("/static", "./static").show_files_listing())
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
