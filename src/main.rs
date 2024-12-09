use actix_web::{web, App, HttpServer, HttpResponse};
use actix_multipart::Multipart;
use futures_util::StreamExt as _;
use std::io::Write;
use std::fs;
use tera::Tera;
use sanitize_filename::sanitize;
use std::fs::File;

async fn upload_file(mut payload: Multipart) -> HttpResponse {
    while let Some(Ok(mut field)) = payload.next().await {
        // Récupère le nom de fichier à partir de ContentDisposition
        if let Some(filename) = field.content_disposition().get_filename() {
            let filepath = format!("./static/uploads/{}", sanitize(&filename));

            // Crée le fichier
            let mut file = match File::create(&filepath) {
                Ok(f) => f,
                Err(_) => return HttpResponse::InternalServerError().body("Erreur lors de la création du fichier"),
            };

            // Écrit les données dans le fichier chunk par chunk
            while let Some(Ok(chunk)) = field.next().await {
                if let Err(_) = file.write_all(&chunk) {
                    return HttpResponse::InternalServerError().body("Erreur lors de l'écriture dans le fichier");
                }
            }
        }
    }

    HttpResponse::Ok().body("Fichier uploadé avec succès !")
}




// 📂 Fonction pour lister les fichiers
async fn list_files() -> HttpResponse {
    let mut file_list = vec![];

    if let Ok(entries) = fs::read_dir("./static/uploads/") {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_name) = entry.file_name().into_string() {
                    file_list.push(file_name);
                }
            }
        }
    }

    HttpResponse::Ok().json(file_list)
}

// 📁 Fonction pour télécharger un fichier
async fn download_file(file_name: web::Path<String>) -> HttpResponse {
    let file_path = format!("./static/uploads/{}", file_name.into_inner());
    if let Ok(file_content) = fs::read(file_path) {
        HttpResponse::Ok()
            .content_type("application/octet-stream")
            .body(file_content)
    } else {
        HttpResponse::NotFound().body("Fichier non trouvé")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 📂 Crée les dossiers nécessaires s'ils n'existent pas
    if !std::path::Path::new("./static/uploads/").exists() {
        std::fs::create_dir_all("./static/uploads/").unwrap();
    }

    println!("🚀 Serveur lancé sur http://127.0.0.1:8080");
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(Tera::new("templates/*.html").unwrap()))
            .route("/", web::get().to(index)) // Page principale
            .route("/upload", web::post().to(upload_file)) // Upload de fichiers
            .route("/files", web::get().to(list_files)) // Liste des fichiers
            .route("/download/{file_name}", web::get().to(download_file)) // Télécharger un fichier
            .service(actix_files::Files::new("/static", "./static")) // Dossier statique
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

// 📄 Fonction pour afficher la page principale
async fn index(tmpl: web::Data<Tera>) -> HttpResponse {
    let mut ctx = tera::Context::new();
    if let Ok(entries) = fs::read_dir("./static/uploads/") {
        let files: Vec<String> = entries
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect();
        ctx.insert("files", &files);
    }

    let rendered = tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}
