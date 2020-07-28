#[macro_use]
extern crate clap;
extern crate crypto;
extern crate futures;
extern crate futures_core;
extern crate futures_util;
extern crate reqwest;
extern crate serde_json;
extern crate tokio;

use bytes::Bytes;
use clap::App;
use crypto::digest::Digest;
use futures_util::StreamExt;

const MANIFEST_URL: &'static str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub latest: Latest,
    pub versions: Vec<Version>,
}

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub url: String,
    pub time: String,
    pub release_time: String,
}

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub id: String,
    pub downloads: Downloads,
    pub main_class: String,
    pub minimum_launcher_version: i64,
    pub release_time: String,
    pub time: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Downloads {
    pub client: Client,
    pub server: Server,
}

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Client {
    pub sha1: String,
    pub size: i64,
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub sha1: String,
    pub size: i64,
    pub url: String,
}

impl Manifest {
    pub fn get(&self, release_type: Option<&str>, version: Option<&str>) -> Option<Version> {
        let mut manifest_version: Option<Version> = None;

        let get_latest: bool = (release_type.is_none() && version.is_none())
            || (release_type.is_some() && version.is_none())
            || (version.is_some() && version.unwrap().eq("latest"));

        for v in &self.versions {
            // short-circuit. They know what they want
            if version.is_some() && version.unwrap().eq(v.id.as_str()) {
                manifest_version = Some(v.clone());
                break;
            }

            if get_latest {
                if release_type.is_some() && release_type.unwrap().eq("snapshot") {
                    if v.id == self.latest.snapshot {
                        manifest_version = Some(v.clone());
                        break;
                    }
                } else {
                    if v.id == self.latest.release {
                        manifest_version = Some(v.clone());
                        break;
                    }
                }
            }
        }

        manifest_version
    }
}

async fn download_jar(file_name: &str, url: &str, sha: &str) {
    let mut stream = reqwest::get(url)
        .await
        .expect("Failed to download jar file")
        .bytes_stream();

    {
        use crypto::sha1::Sha1;
        use std::fs::OpenOptions;
        use std::io::prelude::*;

        let mut hasher: Sha1 = Sha1::new();

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(file_name)
            .expect(format!("Failed to open {}", file_name).as_str());

        while let Some(block) = stream.next().await {
            let b: Bytes = block.expect("Unable to download jar file.");
            hasher.input(b.as_ref());
            file.write(b.as_ref())
                .expect(format!("Failed to write to file {}", file_name).as_str());
        }

        if sha != hasher.result_str() {
            panic!("Shasum check failed, please retry the download.")
        }
    }
}

fn main() {
    let yaml = load_yaml!("args.yml");
    let matches = App::from_yaml(yaml)
        .version(crate_version!())
        .about(crate_description!())
        .get_matches();

    let version = matches.value_of("minecraft_version");
    let release_type = matches.value_of("release_type");

    let manifest: Manifest = reqwest::blocking::get(MANIFEST_URL)
        .expect("Failed to fetch manifest")
        .json::<Manifest>()
        .expect("Failed to parse json manifest, please file a bug report.");

    let minecraft_version: Version = manifest
        .get(release_type, version)
        .expect(format!("Version {:?} was not found in manifest", version).as_ref());

    println!("Found Minecraft version {:?}", minecraft_version.id);
    let versioned_manifest: Release = reqwest::blocking::get(&minecraft_version.url)
        .expect("failed to download version manifest")
        .json::<Release>()
        .expect("Failed to parse release json manifest, please file a bug report.");

    println!(
        "Downloading {} bytes from {}",
        versioned_manifest.downloads.server.size, versioned_manifest.downloads.server.url
    );
    let file_name = match matches.value_of("output") {
        None => format!("minecraft_server_{}.jar", minecraft_version.id),
        Some(name) => name.to_string(),
    };

    tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime.")
        .block_on(download_jar(
            file_name.as_str(),
            versioned_manifest.downloads.server.url.as_str(),
            versioned_manifest.downloads.server.sha1.as_str(),
        ));
}

#[cfg(test)]
mod tests {
    use crate::{Latest, Manifest, Version};

    #[test]
    fn get_version_no_args() {
        let manifest: Manifest = test_manifest();
        let expected: Version = Version {
            id: "1.15.2".to_string(),
            type_field: "release".to_string(),
            url: "".to_string(),
            time: "".to_string(),
            release_time: "".to_string(),
        };
        let actual: Option<Version> = manifest.get(None, None);
        assert!(actual.is_some());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn get_version_with_version_latest() {
        let manifest: Manifest = test_manifest();
        let expected: Version = Version {
            id: "1.15.2".to_string(),
            type_field: "release".to_string(),
            url: "".to_string(),
            time: "".to_string(),
            release_time: "".to_string(),
        };
        let actual: Option<Version> = manifest.get(None, Some("latest"));
        assert!(actual.is_some());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn get_version_with_release_latest() {
        let manifest: Manifest = test_manifest();
        let expected: Version = Version {
            id: "1.15.2".to_string(),
            type_field: "release".to_string(),
            url: "".to_string(),
            time: "".to_string(),
            release_time: "".to_string(),
        };
        let actual: Option<Version> = manifest.get(Some("release"), None);
        assert!(actual.is_some());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn get_version_with_release_latest_snapshot() {
        let manifest: Manifest = test_manifest();
        let expected: Version = Version {
            id: "1.16-pre2".to_string(),
            type_field: "snapshot".to_string(),
            url: "".to_string(),
            time: "".to_string(),
            release_time: "".to_string(),
        };
        let actual: Option<Version> = manifest.get(Some("snapshot"), None);
        assert!(actual.is_some());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn get_version_with_release_latest_version_latest() {
        let manifest: Manifest = test_manifest();
        let expected: Version = Version {
            id: "1.15.2".to_string(),
            type_field: "release".to_string(),
            url: "".to_string(),
            time: "".to_string(),
            release_time: "".to_string(),
        };
        let actual: Option<Version> = manifest.get(Some("release"), Some("latest"));
        assert!(actual.is_some());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn get_version_with_release_latest_release_version_latest_snapshot() {
        let manifest: Manifest = test_manifest();
        let expected: Version = Version {
            id: "1.16-pre2".to_string(),
            type_field: "snapshot".to_string(),
            url: "".to_string(),
            time: "".to_string(),
            release_time: "".to_string(),
        };
        let actual: Option<Version> = manifest.get(Some("release"), Some("1.16-pre2"));
        assert!(actual.is_some());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn get_version_with_release_latest_snapshot_version_latest_release() {
        let manifest: Manifest = test_manifest();
        let expected: Version = Version {
            id: "1.15.2".to_string(),
            type_field: "release".to_string(),
            url: "".to_string(),
            time: "".to_string(),
            release_time: "".to_string(),
        };
        let actual: Option<Version> = manifest.get(Some("snapshot"), Some("1.15.2"));
        assert!(actual.is_some());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn get_version_with_version_args() {
        let manifest: Manifest = test_manifest();
        let expected: Version = Version {
            id: "1.15.2".to_string(),
            type_field: "release".to_string(),
            url: "".to_string(),
            time: "".to_string(),
            release_time: "".to_string(),
        };
        let actual: Option<Version> = manifest.get(None, Some("1.15.2"));
        assert!(actual.is_some());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn get_version_with_version_args_lower() {
        let manifest: Manifest = test_manifest();
        let expected: Version = Version {
            id: "1.14.4".to_string(),
            type_field: "release".to_string(),
            url: "".to_string(),
            time: "".to_string(),
            release_time: "".to_string(),
        };
        let actual: Option<Version> = manifest.get(None, Some("1.14.4"));
        assert!(actual.is_some());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn get_version_with_version_args_lower_snapshot() {
        let manifest: Manifest = test_manifest();
        let expected: Version = Version {
            id: "1.14-pre7".to_string(),
            type_field: "snapshot".to_string(),
            url: "".to_string(),
            time: "".to_string(),
            release_time: "".to_string(),
        };
        let actual: Option<Version> = manifest.get(None, Some("1.14-pre7"));
        assert!(actual.is_some());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn get_version_with_version_args_does_not_exist() {
        let manifest: Manifest = test_manifest();
        let actual: Option<Version> = manifest.get(None, Some("foobar"));
        assert!(actual.is_none());
    }

    fn test_manifest() -> Manifest {
        Manifest {
            latest: Latest {
                release: "1.15.2".to_string(),
                snapshot: "1.16-pre2".to_string(),
            },
            versions: vec![
                Version {
                    id: "1.16-pre2".to_string(),
                    type_field: "snapshot".to_string(),
                    url: "".to_string(),
                    time: "".to_string(),
                    release_time: "".to_string(),
                },
                Version {
                    id: "1.16-pre1".to_string(),
                    type_field: "snapshot".to_string(),
                    url: "".to_string(),
                    time: "".to_string(),
                    release_time: "".to_string(),
                },
                Version {
                    id: "1.15.2".to_string(),
                    type_field: "release".to_string(),
                    url: "".to_string(),
                    time: "".to_string(),
                    release_time: "".to_string(),
                },
                Version {
                    id: "1.14.4".to_string(),
                    type_field: "release".to_string(),
                    url: "".to_string(),
                    time: "".to_string(),
                    release_time: "".to_string(),
                },
                Version {
                    id: "1.14-pre7".to_string(),
                    type_field: "snapshot".to_string(),
                    url: "".to_string(),
                    time: "".to_string(),
                    release_time: "".to_string(),
                },
            ],
        }
    }
}
