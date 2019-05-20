#[macro_use]
extern crate clap;
extern crate crypto;
extern crate reqwest;
extern crate serde_json;

use clap::{App, Arg};
use crypto::digest::Digest;
use std::io::Read;

const MANIFEST_URL: &'static str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

fn main() -> std::result::Result<(), Box<std::error::Error>> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(
            Arg::with_name("minecraft_version")
                .short("v")
                .long("version")
                .value_name("MINECRAFT_VERSION")
                .env("MINECRAFT_VERSION")
                .default_value("latest")
                .help("the server.jar version to download")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("OUTPUT")
                .env("MINECRAFT_FILE")
                .help("where to save server.jar")
                .takes_value(true),
        )
        .get_matches();

    let mut version = matches.value_of("minecraft_version").unwrap().to_string();
    let mut version_manifest_url: Option<String> = None;

    {
        let resp = reqwest::get(MANIFEST_URL)?.text()?;
        let v: serde_json::Value = match serde_json::from_str(resp.as_str()) {
            Ok(json) => json,
            Err(e) => panic!(e),
        };

        if version == "latest" {
            version = v["latest"]["release"]
                .as_str()
                .expect("Could not parse manifest")
                .to_string();
        }

        for v in v["versions"].as_array().expect("Could not parse manifest") {
            let ver: String = v["id"]
                .as_str()
                .expect("Could not parse manifest")
                .to_string();
            if ver == version {
                version_manifest_url = Some(
                    v["url"]
                        .as_str()
                        .expect("Could not parse manifest")
                        .to_string(),
                );
                break;
            }
        }
    }

    {
        let url: String;
        match version_manifest_url {
            None => panic!(format!("Version {} was not found in manifest", version)),
            Some(u) => url = u,
        }

        println!("found Minecraft version {}", version);
        let versioned_manifest_resp: String = reqwest::get(url.as_str())?.text()?;
        let v: serde_json::Value = match serde_json::from_str(versioned_manifest_resp.as_str()) {
            Ok(json) => json,
            Err(e) => panic!(e),
        };

        let server_sha1sum = v["downloads"]["server"]["sha1"]
            .as_str()
            .expect("Could not parse manifest");
        let server_size = v["downloads"]["server"]["size"]
            .as_u64()
            .expect("Could not parse manifest");
        let server_url = v["downloads"]["server"]["url"]
            .as_str()
            .expect("Could not parse manifest");

        println!("Downloading {} bytes from {}", server_size, server_url);
        let mut server_jar_response = reqwest::get(server_url)?;

        let mut hasher = crypto::sha1::Sha1::new();
        let mut buf: Vec<u8> = Vec::new();
        server_jar_response.read_to_end(&mut buf)?;

        hasher.input(&mut buf);
        let hex = hasher.result_str();
        assert_eq!(hex, server_sha1sum);

        let file_name = match matches.value_of("output") {
            None => format!("minecraft_server_{}.jar", version),
            Some(name) => name.to_string(),
        };
        std::fs::write(file_name, buf).expect("Unable to save jar file to disk. Out of space?");
    }

    Ok(())
}
