use gmi::{gemtext::GemtextNode, protocol::StatusCode, url::Url};
use std::env;
use warp::{self, http::Response, hyper::Body, Filter};

#[macro_use]
extern crate lazy_static;

// TODO: Consider also formatting "plain/text"
// TODO: Redirects
// TODO: Inputs
// TODO: Serve favicon?

lazy_static! {
    static ref URL: String = env::var("TG2H_URL").unwrap();
    static ref STYLE: String = env::var("TG2H_STYLE").unwrap_or_else(|_| String::new());
}

#[tokio::main]
async fn main() {
    // Solve URL and STYLE so it crashes now if it has to
    let _ = &*URL;
    let _ = &*STYLE;

    let filter = warp::any()
        .and(warp::path::full())
        .map(|p: warp::path::FullPath| proxy(p.as_str()));

    let (_addr, fut) =
        warp::serve(filter).bind_with_graceful_shutdown(([0, 0, 0, 0], 8080), async move {
            println!("tg2h up and running");
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen to shutdown signal");
        });
    fut.await;
}

macro_rules! clean {
    ($expre: expr) => {
        html_escape::encode_text($expre)
    };
}

macro_rules! no_good {
    ($expre: expr) => {
        let aux = $expre;
        return Response::builder()
            .status(500)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Body::from(format!("tg2h: {}", clean!(&aux))))
            .unwrap();
    };
}

// Get Gemini response
fn proxy(path: &str) -> warp::reply::Response {
    let url = format!("{}{}", &*URL, path);
    let Ok(url) = Url::try_from(url.as_str()) else {
        no_good!(format!("bad url: {}", url));
    };

    let Ok(resp) = gmi::request::make_request(&url) else {
        no_good!("could not make request to server");
    };

    match resp.status {
        StatusCode::Input(_) => {
            no_good!(format!("input not supported: {}", resp.meta));
        }
        StatusCode::Success(_) => { /* ok! */ }
        StatusCode::Redirect(_) => {
            no_good!(format!("redirect not supported: {}", resp.meta));
        }
        StatusCode::PermanentFailure(1) => {
            // Error 51 (not found)
            return Response::builder()
                .status(404)
                .header("Content-Type", "text/html; charset=utf-8")
                .body(Body::from(format!("not found: {}", clean!(&resp.meta))))
                .unwrap();
        }
        // Anything else is an actual error
        x => {
            no_good!(format!("error: {:?}: {}", x, resp.meta));
        }
    };

    let mime = resp.meta;
    let raw = resp.data;

    // Gemtext?
    if mime == "text/gemini" {
        // Convert to HTML
        let Ok(text) = String::from_utf8(raw) else {
            no_good!("invalid UTF-8 character in text/gemini mime type");
        };
        Response::builder()
            .status(200)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Body::from(gem2html(&text)))
            .unwrap()
    } else {
        // Not gemtext, return it in its content type
        Response::builder()
            .status(200)
            .header("Content-Type", mime)
            .body(Body::from(raw))
            .unwrap()
    }
}

// Gemtext to HTML
fn gem2html(gem: &str) -> String {
    // Construct the body first
    let nodes = gmi::gemtext::parse_gemtext(gem);
    let mut body = String::new();
    for i in nodes {
        let line = match i {
            GemtextNode::Text(x) => format!("<p>{}</p>", clean!(&x)),
            GemtextNode::Link(x, c) => {
                let c = match c {
                    Some(k) => k,
                    None => x.clone(),
                };
                format!("<a href=\"{}\">{}</a><br>", clean!(&x), clean!(&c))
            }
            GemtextNode::Heading(x) => format!("<h1>{}</h1>", clean!(&x)),
            GemtextNode::SubHeading(x) => format!("<h2>{}</h2>", clean!(&x)),
            GemtextNode::SubSubHeading(x) => format!("<h3>{}</h3>", clean!(&x)),
            GemtextNode::ListItem(x) => format!("<li>{}</li>", clean!(&x)),
            GemtextNode::Blockquote(x) => format!("<pre>{}</pre>", clean!(&x)),
            GemtextNode::Preformatted(x, _) => format!("<pre>{}</pre>", clean!(&x)),
            GemtextNode::EmptyLine => "<br>".to_string(),
        } + "\n";
        body += &line;
    }

    // Do we have a title?
    let mut title = String::new();
    if let Some(o) = gem.lines().next() {
        if o.len() > 2 {
            let (lhs, rhs) = o.split_at(2);
            if lhs == "# " {
                title = format!("<title>{}</title>", rhs);
            }
        }
    }

    // Do we have a style?
    let style = if STYLE.len() > 0 {
        format!("<link rel=\"stylesheet\" href=\"{}\">", *STYLE)
    } else {
        String::new()
    };

    // Wrap it in a nice HTML file with a header, and that's it
    format!(
        "
<!DOCTYPE HTML>

<HTML>
    <head>
        <meta charset=\"UTF-8\">
        <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />
        {}
        {}
    </head>
    <body>
        <div id='outer'><div id='middle'><div id='inner'>
        {}
        </div></div></div>
    </body>
</HTML>",
        title, style, body
    )
}
