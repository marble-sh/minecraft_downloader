extern crate clap;
extern crate lazy_static;

use std::str::FromStr;
use std::sync::Mutex;

use clap::{App, Arg, arg_enum, crate_description, crate_name, crate_version};
use lazy_static::lazy_static;

arg_enum! {
    #[allow(non_camel_case_types)]
    pub enum ReleaseType {
        release,
        snapshot
    }
}

arg_enum! {
    #[allow(non_camel_case_types)]
    pub enum JarType {
        server,
        client
    }
}

lazy_static! {
    static ref VERSION: Mutex<String> = Mutex::new(String::new());
    static ref RELEASE_TYPE: Mutex<String> = Mutex::new(String::new());
    static ref JAR_TYPE: Mutex<String> = Mutex::new(String::new());
    static ref OUTPUT: Mutex<Option<String>> = Mutex::new(None);
    static ref NO_DOWNLOAD: Mutex<bool> = Mutex::new(false);
    static ref NO_OVERWRITE: Mutex<bool> = Mutex::new(false);
    static ref QUIET: Mutex<bool> = Mutex::new(false);
}

pub fn version() -> String {
    VERSION.lock().unwrap().to_string()
}

pub fn release_type() -> ReleaseType {
    let release_type = RELEASE_TYPE.lock().unwrap().to_string();
    ReleaseType::from_str(release_type.as_str()).unwrap()
}

pub fn jar_type() -> JarType {
    let jar_type = JAR_TYPE.lock().unwrap().to_string();
    JarType::from_str(jar_type.as_str()).unwrap()
}

pub fn output() -> Option<String> {
    let output = OUTPUT.lock().unwrap();
    return if output.is_some() {
        Some(output.as_ref().unwrap().clone())
    } else {
        None
    };
}

pub fn no_download() -> bool {
    *NO_DOWNLOAD.lock().unwrap()
}

pub fn no_overwrite() -> bool {
    *NO_OVERWRITE.lock().unwrap()
}

pub fn quiet() -> bool {
    *QUIET.lock().unwrap()
}

pub fn init() {
    let args = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            Arg::with_name("no_download")
                .short('n')
                .long("no-download")
                .takes_value(false)
                .help("Do not download the latest .jar, instead output only the found version.")
        )
        .arg(
            Arg::with_name("no_overwrite")
                .short('x')
                .long("no-overwrite")
                .takes_value(false)
                .help("Do not overwrite the file on disk.")
        )
        .arg(
            Arg::with_name("quiet")
                .short('q')
                .long("quiet")
                .takes_value(false)
                .help("Silence everything but errors and necessary output.")
        )
        .arg(
            Arg::with_name("jar_type")
                .short('j')
                .long("jar-type")
                .env("MINECRAFT_JAR_TYPE")
                .takes_value(true)
                .value_name("JAR_TYPE")
                .possible_values(&JarType::variants())
                .default_value("server")
                .help("The type of jar to download.")
        )
        .arg(
            Arg::with_name("minecraft_version")
                .short('v')
                .long("version")
                .env("MINECRAFT_VERSION")
                .takes_value(true)
                .value_name("VERSION")
                .default_value("latest")
                .help("The specific version to download. Takes precedence over release_type.")
        )
        .arg(
            Arg::with_name("output")
                .short('o')
                .long("output")
                .env("MINECRAFT_OUTPUT")
                .takes_value(true)
                .value_name("OUTPUT")
                .help("Where to save the jar file. Overwrites the file if it already exists on disk.")
        )
        .arg(
            Arg::with_name("release_type")
                .short('t')
                .long("type")
                .env("MINECRAFT_RELEASE_TYPE")
                .takes_value(true)
                .value_name("RELEASE_TYPE")
                .possible_values(&ReleaseType::variants())
                .default_value("release")
                .help("The type of release to download, only used if version is \"latest\".")
        )
        .get_matches();

    *VERSION.lock().unwrap() = args.value_of("minecraft_version").unwrap_or_default().to_string();
    *RELEASE_TYPE.lock().unwrap() = args.value_of("release_type").unwrap_or_default().to_string();
    *JAR_TYPE.lock().unwrap() = args.value_of("jar_type").unwrap_or_default().to_string();
    *NO_DOWNLOAD.lock().unwrap() = args.is_present("no_download");
    *NO_OVERWRITE.lock().unwrap() = args.is_present("no_overwrite");
    *QUIET.lock().unwrap() = args.is_present("quiet");

    let output = args.value_of("output");
    *OUTPUT.lock().unwrap() = if output.is_some() {
        Some(output.unwrap().to_string())
    } else {
        None
    }
}