use rocket::Request;
use rocket_dyn_templates::{context, Template};

#[catch(404)]
pub async fn not_found(req: &Request<'_>) -> Template {
    Template::render("404", context! {
        uri: req.uri(),
        title: "404 Not Found",
        lang: "en",
    })
}