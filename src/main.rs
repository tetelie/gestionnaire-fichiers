use actix_multipart::Multipart;
use actix_web::{web, App, HttpServer, HttpResponse};
use futures_util::StreamExt as _;
use sanitize_filename::sanitize;
use std::fs::{self, File};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

async fn upload_file(mut payload: Multipart) -> HttpResponse {
    while let Some(Ok(mut field)) = payload.next().await {
        if let Some(filename) = field.content_disposition().get_filename() {
            let filepath = format!("./static/uploads/{}", sanitize(filename));

            let mut file = match File::create(&filepath) {
                Ok(f) => f,
                Err(_) => return HttpResponse::InternalServerError().body("Erreur lors de la création du fichier"),
            };

            while let Some(Ok(chunk)) = field.next().await {
                if let Err(_) = file.write_all(&chunk) {
                    return HttpResponse::InternalServerError().body("Erreur lors de l'écriture dans le fichier");
                }
            }
        }
    }

    HttpResponse::Ok().body("Fichier uploadé avec succès !")
}

async fn list_files() -> HttpResponse {
    let mut files_info = vec![];

    if let Ok(entries) = fs::read_dir("./static/uploads/") {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if let Ok(metadata) = entry.metadata() {
                        let file_size = metadata.len();
                        let created_time = metadata.created().unwrap_or(SystemTime::now());
                        let modified_time = metadata.modified().unwrap_or(SystemTime::now());

                        files_info.push((
                            file_name,
                            file_size,
                            format_time(created_time),
                            format_time(modified_time),
                        ));
                    }
                }
            }
        }
    }

    HttpResponse::Ok().json(files_info)
}

async fn delete_file(file_name: web::Path<String>) -> HttpResponse {
    let file_path = format!("./static/uploads/{}", sanitize(file_name.as_str()));

    if fs::remove_file(&file_path).is_ok() {
        HttpResponse::Ok().body("Fichier supprimé avec succès !")
    } else {
        HttpResponse::NotFound().body("Fichier non trouvé")
    }
}

async fn download_file(file_name: web::Path<String>) -> HttpResponse {
    let file_path = format!("./static/uploads/{}", sanitize(file_name.as_str()));

    match fs::read(&file_path) {
        Ok(file_content) => {
            HttpResponse::Ok()
                .content_type("application/octet-stream")
                .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", file_name)))
                .body(file_content)
        }
        Err(_) => HttpResponse::NotFound().body("Fichier non trouvé"),
    }
}

fn format_time(system_time: SystemTime) -> String {
    let datetime = system_time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let naive_datetime = chrono::NaiveDateTime::from_timestamp(datetime as i64, 0);
    naive_datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

async fn index(tmpl: web::Data<tera::Tera>) -> HttpResponse {
    let mut ctx = tera::Context::new();
    if let Ok(entries) = fs::read_dir("./static/uploads/") {
        let mut files = vec![];
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if let Ok(metadata) = entry.metadata() {
                        files.push((
                            file_name.clone(),
                            metadata.len(), // Taille du fichier
                            format_time(metadata.created().unwrap_or(SystemTime::now())), // Date d'upload
                            format_time(metadata.modified().unwrap_or(SystemTime::now())), // Date de modification
                        ));
                    }
                }
            }
        }
        ctx.insert("files", &files);
    }

    let rendered = tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if !std::path::Path::new("./static/uploads/").exists() {
        std::fs::create_dir_all("./static/uploads/").unwrap();
    }

    println!("🚀 Serveur lancé sur http://127.0.0.1:8080");
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(tera::Tera::new("templates/*.html").unwrap()))
            .route("/", web::get().to(index))
            .route("/upload", web::post().to(upload_file))
            .route("/files", web::get().to(list_files))
            .route("/delete/{file_name}", web::post().to(delete_file))
            .route("/download/{file_name}", web::get().to(download_file)) // Route de téléchargement
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
