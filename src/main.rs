use std::path::PathBuf;

use bili_live_chat::config::Config;
use bili_live_chat::App;
use bili_live_chat::Credential;
use clap::{arg, command, value_parser};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (room_id, config) = cli_init();
    let mut app = App::new(room_id, config).await;

    app.run().await;

    Ok(())
}

fn cli_init() -> (u32, Config) {
    let matches = command!()
        .arg(
            arg!(
                -c --config <FILE> "Sets loading path for config file"
            )
            .required(false)
            .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(
                -d <ROOM_ID> "Specify a live room"
            )
            .required(true),
        )
        .arg(
            arg!(
                -s --sessdata <SESSDATA>
            )
            .required(false),
        )
        .arg(
            arg!(
                -b --bili_jct <BILI_JCT>
            )
            .required(false),
        )
        .arg(
            arg!(
                -u --buvid3 <BUVID3>
            )
            .required(false),
        )
        .get_matches();

    let room_id = matches
        .get_one::<String>("ROOM_ID")
        .unwrap()
        .parse::<u32>()
        .unwrap_or_else(|_| panic!("Invalid Room Id."));

    let mut config = if let Some(path) = matches.get_one::<PathBuf>("config") {
        let path_str = path.to_str().unwrap();
        Config::from_file(path_str).unwrap_or_else(|_| panic!("No such file or directory"))
    } else if let Ok(conf) = Config::from_file(
        format!(
            "{}/.config/bili-live-chat/config.yml",
            directories::BaseDirs::new()
                .unwrap()
                .home_dir()
                .to_str()
                .unwrap()
        )
        .as_str(),
    ) {
        conf
    } else {
        match (
            matches.get_one::<String>("sessdata"),
            matches.get_one::<String>("bili_jct"),
            matches.get_one::<String>("buvid3"),
        ) {
            (Some(sessdata), Some(bili_jct), Some(buvid3)) => {
                let conf = Config {
                    credential: Credential {
                        session_data: sessdata.to_owned(),
                        bili_jct: bili_jct.to_owned(),
                        buvid3: buvid3.to_owned(),
                    },
                };

                conf
            }
            _ => panic!("\"~/.config/bili-live-chat/config.yml\" does not exist."),
        }
    };

    if let Some(sessdata) = matches.get_one::<String>("sessdata") {
        config.credential.session_data = sessdata.clone();
    }

    if let Some(bili_jct) = matches.get_one::<String>("bili_jct") {
        config.credential.bili_jct = bili_jct.clone();
    }

    if let Some(buvid3) = matches.get_one::<String>("buvid3") {
        config.credential.buvid3 = buvid3.clone();
    }

    (room_id, config)
}
