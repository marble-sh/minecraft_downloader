extern crate clap;
extern crate crypto;
extern crate reqwest;
extern crate serde_json;

use std::fmt::{Display, Formatter};
use clap::{Parser, ValueEnum};
use serde_derive::{Deserialize, Serialize};

const MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
const BUG_REPORT_URL: &str = "https://github.com/marble-sh/minecraft_downloader/issues/new";

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[allow(non_camel_case_types)]
enum ReleaseType {
    release,
    snapshot
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[allow(non_camel_case_types)]
enum JarType {
    server,
    client
}

impl Display for ReleaseType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ReleaseType::release => write!(f, "release"),
            ReleaseType::snapshot => write!(f, "snapshot")
        }
    }
}
impl Display for JarType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            JarType::server => write!(f, "server"),
            JarType::client => write!(f, "client")
        }
    }
}

#[derive(Parser)]
#[command(about, author, long_about = None)]
struct Args {
    #[arg(short, long, long_help = "The specific version to download")]
    version: Option<String>,

    #[arg(short, long, default_value_t = ReleaseType::release, long_help = "The type of release to download")]
    release_type: ReleaseType,

    #[arg(short, long, default_value_t = JarType::server, long_help = "The type of jar to download")]
    jar_type: JarType,

    #[arg(short, long, long_help = "The path to save the jar file to")]
    output: Option<String>,

    #[arg(long, default_value_t = false, long_help = "Do not download the latest .jar, instead output only the found version")]
    no_download: bool,

    #[arg(long, default_value_t = false, long_help = "Do not overwrite the file on disk")]
    no_overwrite: bool,

    #[arg(short, long, default_value_t = false, long_help = "Silence everything but errors and necessary output")]
    quiet: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub latest: Latest,
    pub versions: Vec<Version>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub url: String,
    pub time: String,
    pub release_time: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Downloads {
    pub client: Client,
    pub server: Server,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Client {
    pub sha1: String,
    pub size: i64,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub sha1: String,
    pub size: i64,
    pub url: String,
}

fn print(quiet: bool, msg: String) {
    if !quiet {
        println!("{}", msg);
    }
}

impl Version {
    fn copy(&self) -> Self {
        Self {
            id: self.id.to_string(),
            type_field: self.type_field.to_string(),
            url: self.url.to_string(),
            time: self.time.to_string(),
            release_time: self.release_time.to_string()
        }
    }
}

impl Manifest {
    fn find_by_id(&mut self, id: &str) -> Option<Version> {
        self.versions.sort_by_key(|probe| probe.id.clone());
        let result = &self.versions.binary_search_by_key(&id, |probe| probe.id.as_str());

        if result.is_ok() {
            let v = &self.versions.get(result.unwrap()).unwrap();
            Some(v.copy())
        } else {
            None
        }
    }

    fn get(&mut self, release_type: ReleaseType, version: &str) -> Option<Version> {
        if version.ne("latest") {
            return self.find_by_id(version);
        }

        match release_type {
            ReleaseType::release => {
                let latest = self.latest.release.clone();
                self.find_by_id(latest.as_str())
            }
            ReleaseType::snapshot => {
                let latest = self.latest.snapshot.clone();
                self.find_by_id(latest.as_str())
            }
        }
    }
}

fn bug_report_url(title: &str, tag: &str) -> String {
    extern crate url;
    use url::form_urlencoded;

    let tags = format!("bug,{}", tag);

    form_urlencoded::Serializer::new(BUG_REPORT_URL.to_string())
        .append_pair("assignees", "marble-sh")
        .append_pair("labels", tags.as_str())
        .append_pair("template", "bug_report.md")
        .append_pair("title", title)
        .finish()
}

fn download_jar(file_name: &str, url: &str, sha: &str, no_overwrite: bool) {
    use bytes::buf::Buf;
    use crypto::digest::Digest;
    use crypto::sha1::Sha1;
    use reqwest::blocking::Response;

    let resp: Result<Response, reqwest::Error> = reqwest::blocking::get(url);
    if resp.is_err() {
        let error = resp.as_ref().err().unwrap().to_string();
        panic!("Failed to fetch jar file, {}", error);
    }

    let bytes = resp.unwrap().bytes();
    if bytes.is_err() {
        let error = bytes.as_ref().err().unwrap().to_string();
        panic!("Failed to unwrap jar file, {}", error);
    }

    let jar: &[u8] = &*bytes.as_ref().unwrap();

    let mut hasher: Sha1 = Sha1::new();
    hasher.input(jar);
    if sha != hasher.result_str() {
        panic!("Checksum mismatch, please retry the download.");
    }

    let mut options = std::fs::OpenOptions::new();
    if no_overwrite {
        options.create_new(true);
    } else {
        options.create(true);
    }

    let file: Result<std::fs::File, std::io::Error> = options
        .write(true)
        .append(false)
        .open(file_name);

    if file.is_err() {
        let error: String = file.as_ref().err().unwrap().to_string();
        panic!("Failed to open {}\n{}", file_name, error);
    }

    let result: Result<u64, std::io::Error> = std::io::copy(&mut bytes.unwrap().reader(), &mut file.unwrap());

    if result.is_err() {
        let error: String = result.as_ref().err().unwrap().to_string();
        panic!("Failed to write to disk, {}", error);
    }
}

fn get_manifest() -> Manifest {
    use reqwest::blocking::Response;

    let manifest_result: Result<Response, reqwest::Error> = reqwest::blocking::get(MANIFEST_URL);
    if manifest_result.is_err() {
        let error = manifest_result.as_ref().err().unwrap().to_string();
        panic!("Failed to fetch manifest, {}", error);
    }

    let manifest: Result<Manifest, reqwest::Error> = manifest_result.unwrap().json::<Manifest>();
    if manifest.is_err() {
        let error = manifest.as_ref().err().unwrap().to_string();
        let title = format!("Invalid Manifest - {}", error);
        let url = bug_report_url(title.as_str(), "manifest");
        panic!("Failed to parse json manifest, {}\n\nPlease file a bug report.\n{}", error, url);
    }

    return manifest.unwrap();
}

fn get_release(version: &Version) -> Release {
    use reqwest::blocking::Response;

    let versioned_manifest_result: Result<Response, reqwest::Error> = reqwest::blocking::get(&version.url);
    if versioned_manifest_result.is_err() {
        let error = versioned_manifest_result.as_ref().err().unwrap().to_string();
        panic!("Failed to download version manifest, {}", error);
    }

    let versioned_manifest: Result<Release, reqwest::Error> = versioned_manifest_result.unwrap().json::<Release>();
    if versioned_manifest.is_err() {
        let error = versioned_manifest.as_ref().err().unwrap().to_string();
        let title = format!("Invalid Versioned Manifest - {}", error);
        let url = bug_report_url(title.as_str(), "manifest");
        panic!("Failed to parse json manifest, {}\n\nPlease file a bug report.\n{}", error, url);
    }

    return versioned_manifest.unwrap();
}

fn main() {
    let args = Args::parse();
    let version = args.version.unwrap_or("latest".to_string());

    // Fetch the json file and look for the version
    let mut manifest = get_manifest();

    let minecraft_version: Version = match manifest.get(args.release_type, version.as_str()) {
        None => panic!("Version \"{}\" was not found in manifest", version),
        Some(v) => v
    };

    if args.no_download {
        println!("{}", minecraft_version.id);
        std::process::exit(0);
    }

    print(args.quiet, format!("Found Minecraft version {}", minecraft_version.id));
    let release = get_release(&minecraft_version);

    let file_name = match args.output {
        None => format!("minecraft_{}_{}.jar", args.jar_type, minecraft_version.id),
        Some(name) => name,
    };

    print(args.quiet, format!("Saving jar file as {}", file_name));

    let size;
    let url;
    let sha;
    match args.jar_type {
        JarType::server => {
            size = release.downloads.server.size;
            url = release.downloads.server.url.as_str();
            sha = release.downloads.server.sha1.as_str();
        }
        JarType::client => {
            size = release.downloads.client.size;
            url = release.downloads.client.url.as_str();
            sha = release.downloads.client.sha1.as_str();
        }
    }

    let file_exists: bool = std::path::Path::new(file_name.as_str()).exists();
    if args.no_overwrite && file_exists {
        eprintln!("Refusing to download {}, file already exists", file_name);
        std::process::exit(0);
    }

    print(args.quiet, format!("Downloading {} bytes from {}...", size, url));
    download_jar(
        file_name.as_str(),
        url,
        sha,
        args.no_overwrite,
    );
}

#[cfg(test)]
mod tests {
    use crate::{Latest, Manifest, ReleaseType, Version};

    #[test]
    fn it_returns_the_latest_snapshot_version() {
        let mut manifest: Manifest = test_manifest();
        let expected: Version = Version {
            id: "1.16-pre2".to_string(),
            type_field: "snapshot".to_string(),
            url: "".to_string(),
            time: "".to_string(),
            release_time: "".to_string(),
        };
        let actual: Option<Version> = manifest.get(ReleaseType::snapshot, "latest");
        assert!(actual.is_some());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn it_returns_the_latest_release_version() {
        let mut manifest: Manifest = test_manifest();
        let expected: Version = Version {
            id: "1.15.2".to_string(),
            type_field: "release".to_string(),
            url: "".to_string(),
            time: "".to_string(),
            release_time: "".to_string(),
        };
        let actual: Option<Version> = manifest.get(ReleaseType::release, "latest");
        assert!(actual.is_some());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn it_will_return_a_snapshot_version_regardless_of_release_type() {
        let mut manifest: Manifest = test_manifest();
        let expected: Version = Version {
            id: "1.16-pre2".to_string(),
            type_field: "snapshot".to_string(),
            url: "".to_string(),
            time: "".to_string(),
            release_time: "".to_string(),
        };
        let actual: Option<Version> = manifest.get(ReleaseType::release, "1.16-pre2");
        assert!(actual.is_some());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn it_will_return_a_release_version_regardless_of_release_type() {
        let mut manifest: Manifest = test_manifest();
        let expected: Version = Version {
            id: "1.15.2".to_string(),
            type_field: "release".to_string(),
            url: "".to_string(),
            time: "".to_string(),
            release_time: "".to_string(),
        };
        let actual: Option<Version> = manifest.get(ReleaseType::snapshot, "1.15.2");
        assert!(actual.is_some());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn it_will_return_an_old_release_version_if_asked() {
        let mut manifest: Manifest = test_manifest();
        let expected: Version = Version {
            id: "1.14.4".to_string(),
            type_field: "release".to_string(),
            url: "".to_string(),
            time: "".to_string(),
            release_time: "".to_string(),
        };
        let actual: Option<Version> = manifest.get(ReleaseType::release, "1.14.4");
        assert!(actual.is_some());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn it_will_return_a_lower_snapshot_version_if_asked() {
        let mut manifest: Manifest = test_manifest();
        let expected: Version = Version {
            id: "1.14-pre7".to_string(),
            type_field: "snapshot".to_string(),
            url: "".to_string(),
            time: "".to_string(),
            release_time: "".to_string(),
        };
        let actual: Option<Version> = manifest.get(ReleaseType::snapshot, "1.14-pre7");
        assert!(actual.is_some());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn it_will_return_nothing_if_the_version_asked_does_not_exist() {
        let mut manifest: Manifest = test_manifest();
        let actual: Option<Version> = manifest.get(ReleaseType::release, "1.17.1");
        assert!(actual.is_none());
    }

    #[test]
    fn it_will_return_the_latest_release_version_even_if_a_snapshot_is_requested_if_that_is_the_latest() {
        let mut manifest: Manifest = test_manifest();
        // make the current snapshot the same as teh release
        // this is what returns when the latest version is a release version
        manifest.latest.snapshot = "1.15.2".to_string();

        let actual: Option<Version> = manifest.get(ReleaseType::snapshot, "latest");
        assert!(actual.is_some());

        let expected: Version = Version {
            id: "1.15.2".to_string(),
            type_field: "release".to_string(),
            url: "".to_string(),
            time: "".to_string(),
            release_time: "".to_string(),
        };

        assert_eq!(expected, actual.unwrap());
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