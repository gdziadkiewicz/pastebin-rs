use paste_id::PasteId;
use rocket::{data::ByteUnit, delete, get, http::{uri::Absolute, ContentType, Status}, post, response::status::{self, NoContent}, routes, tokio::fs::File, uri, Data};

mod paste_id;

#[get("/")]
fn index() -> &'static str {
    "
    USAGE

      POST /

          accepts raw data in the body of the request and responds with a URL of
          a page containing the body's content

      GET /<id>

          retrieves the content for the paste with id `<id>`
    "
}

#[get("/<id>")]
async fn retrieve(id: PasteId<'_>) -> (ContentType, Option<File>) {
    (ContentType::Text, File::open(id.file_path()).await.ok())
}

#[delete("/<id>")]
async fn delete(id: PasteId<'_>) -> status::NoContent {
    let _ = rocket::tokio::fs::remove_file(id.file_path()).await;
    status::NoContent
}

// In a real application, these would be retrieved dynamically from a config.
const ID_LENGTH: usize = 3;
const HOST: Absolute<'static> = uri!("http://localhost:8000");

#[post("/", data = "<paste>")]
async fn upload(paste: Data<'_>) ->  std::io::Result<(Status, (ContentType,String))> {
    let ct = ContentType::Text;
    let id = PasteId::new(ID_LENGTH);
    let file = paste.open(ByteUnit::Megabyte(2)).into_file(id.file_path()).await?;
    let status = 
        match file.is_complete() {
            true => Status::Created,
            false => Status::PartialContent,
        };
    let new_url = uri!(HOST, retrieve(id)).to_string();
    Ok((status, (ct, new_url)))
}

async fn clean() -> std::io::Result<()> {
    rocket::tokio::fs::remove_dir_all("upload").await?;
    rocket::tokio::fs::create_dir("upload").await?;
    Ok(())
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _cleaner = rocket::tokio::spawn(async {
        loop {
            rocket::tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            clean().await.unwrap_or_else(|e| println!("Error {:?}", e));
        }
    });
    let _rocket = rocket::build()
        .mount("/", routes![index, retrieve, upload, delete])
        .launch()
        .await?;

    Ok(())
}

// #[launch]
// fn rocket() -> _ {
//     rocket::build().mount("/", routes![index, retrieve, upload, delete])
// }