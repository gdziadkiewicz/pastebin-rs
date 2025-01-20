use paste_id::PasteId;
use rocket::{data::ByteUnit, delete, get, http::{uri::Absolute, ContentType}, launch, post, response::{content::RawText, status::{self, NoContent}}, routes, tokio::fs::File, uri, Data};

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
    status::NoContent
}

// In a real application, these would be retrieved dynamically from a config.
const ID_LENGTH: usize = 3;
const HOST: Absolute<'static> = uri!("http://localhost:8000");

#[post("/", data = "<paste>")]
async fn upload(paste: Data<'_>) ->  std::io::Result<(ContentType,String)> {
    let id = PasteId::new(ID_LENGTH);
    paste.open(ByteUnit::Megabyte(2)).into_file(id.file_path()).await?;
    Ok((ContentType::Text, uri!(HOST, retrieve(id)).to_string()))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, retrieve, upload])
}