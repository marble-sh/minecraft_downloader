#[macro_use]
extern crate clap;
extern crate crypto;
extern crate reqwest;
extern crate serde_json;

use bytes::buf::BufExt;
use bytes::{Buf, Bytes};
use clap::App;
use crypto::digest::Digest;
use serde_derive::{Deserialize, Serialize};

const MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub latest: Latest,
    pub versions: Vec<Version>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub url: String,
    pub time: String,
    pub release_time: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Downloads {
    pub client: Client,
    pub server: Server,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Client {
    pub sha1: String,
    pub size: i64,
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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
                match release_type {
                    Some(release_type) => match release_type {
                        "snapshot" => {
                            if v.id == self.latest.snapshot {
                                manifest_version = Some(v.clone());
                                break;
                            }
                        }
                        "release" => {
                            if v.id == self.latest.release {
                                manifest_version = Some(v.clone());
                                break;
                            }
                        }
                        _ => {
                            manifest_version = Some(v.clone());
                            break;
                        }
                    },
                    None => {
                        if v.id == self.latest.release {
                            manifest_version = Some(v.clone());
                            break;
                        }
                    }
                }
            }
        }

        manifest_version
    }
}

fn download_jar(file_name: &str, url: &str, sha: &str) {
    use crypto::sha1::Sha1;
    use std::{fs::OpenOptions, io::copy};

    let resp: Bytes = reqwest::blocking::get(url)
        .expect("Failed to fetch server jar")
        .bytes()
        .expect("Failed to extract server jar");

    let jar: &[u8] = resp.bytes();

    let mut hasher: Sha1 = Sha1::new();
    hasher.input(jar);
    if sha != hasher.result_str() {
        panic!("Shasum check failed, please retry the download.")
    }

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(false)
        .open(file_name)
        .unwrap_or_else(|_| panic!("Failed to open {}", file_name));

    copy(&mut resp.reader(), &mut file).expect("failed to copy content");
}

fn main() {
    let yaml = load_yaml!("args.yml");
    let args = App::from_yaml(yaml)
        .version(crate_version!())
        .about(crate_description!())
        .get_matches();

    let version = args.value_of("minecraft_version");
    let release_type = args.value_of("release_type");
    let no_download = args.is_present("no_download");

    let manifest: Manifest = reqwest::blocking::get(MANIFEST_URL)
        .expect("Failed to fetch manifest")
        .json::<Manifest>()
        .expect(
            "Failed to parse json manifest, please file a bug report.\n\
        https://github.com/marblenix/minecraft_downloader/issues/new\
        ?assignees=marblenix&labels=bug,manifest&template=bug_report.md&title=Invalid%20Manifest\n",
        );

    let minecraft_version: Version = manifest
        .get(release_type, version)
        .unwrap_or_else(|| panic!("Version {:?} was not found in manifest", version));

    if no_download {
        println!("{}", minecraft_version.id);
        std::process::exit(0);
    }

    println!("Found Minecraft version {:?}", minecraft_version.id);
    let versioned_manifest: Release = reqwest::blocking::get(&minecraft_version.url)
        .expect("failed to download version manifest")
        .json::<Release>()
        .expect(
            "Failed to parse release json manifest, please file a bug report.\n\
        https://github.com/marblenix/minecraft_downloader/issues/new\
        ?assignees=marblenix&labels=bug,manifest&template=bug_report.md&title=Invalid%20Manifest\n",
        );

    let file_name = match args.value_of("output") {
        None => format!("minecraft_server_{}.jar", minecraft_version.id),
        Some(name) => name.to_string(),
    };

    println!("Saving minecraft .jar as {}", file_name);

    println!(
        "Downloading {} bytes from {}...",
        versioned_manifest.downloads.server.size, versioned_manifest.downloads.server.url
    );

    download_jar(
        file_name.as_str(),
        versioned_manifest.downloads.server.url.as_str(),
        versioned_manifest.downloads.server.sha1.as_str(),
    );
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

    #[test]
    fn get_latest_snapshot_when_no_new_snapshot_is_available() {
        let mut manifest: Manifest = test_manifest();
        manifest.latest.snapshot = manifest.latest.release.clone();
        let actual: Option<Version> = manifest.get(Some("snapshot"), Some("latest"));
        assert!(actual.is_some());
        assert_eq!(manifest.latest.release, actual.unwrap().id);
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
