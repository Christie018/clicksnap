use std::{env, path::PathBuf};
use thirtyfour::prelude::*;
use tokio;
use url::Url;

mod apps;
use apps::*;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // parse args
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        return Err(color_eyre::Report::msg(
            "usage: tkl-webtest <action: test|install> <appliance name> <root URL>",
        ));
    }

    let act = match args[1].as_str() {
        "test" => Action::Test,
        "install" => Action::Install,
        x => panic!("unknown action: {}", x),
    };

    let app = &args[2];
    let url = Url::parse(&args[3])?;

    let mut caps = DesiredCapabilities::chrome();
    caps.accept_insecure_certs(true)?;
    let wdurl = match env::var("TKL_WEBDRIVER_URL") {
        Ok(s) => s,
        Err(_) => "http://localhost:4444/".to_string(),
    };
    let wd = WebDriver::new(&wdurl, caps).await?;
    wd.set_window_rect(0, 0, 1366 + 8, 768 + 126).await?; // account for window geometry
    let scrpath = match env::var("TKL_SCREENSHOT_PATH") {
        Ok(s) => s,
        Err(_) => "/tmp".to_string(),
    };

    let preseeds = Preseeds {
        root_pass: env::var("ROOT_PASS").unwrap_or("turnkey".to_owned()),
        db_pass: env::var("DB_PASS").unwrap_or("turnkey".to_owned()),
        app_pass: env::var("APP_PASS").unwrap_or("turnkey".to_owned()),
        app_email: env::var("APP_EMAIL").unwrap_or("admin@example.com".to_owned()),
        app_domain: env::var("APP_DOMAIN").unwrap_or("example.com".to_owned()),
    };
    let st = State {
        wd,
        act,
        url,
        ssp: PathBuf::from(&scrpath),
        pse: preseeds,
    };
    match RUNNERS.get(app.as_str()) {
        Some(t) => t.exec(st).await.map_err(color_eyre::Report::new),
        None => Err(color_eyre::Report::msg(format!("Unknown app: {:?}!", app))),
    }
}
