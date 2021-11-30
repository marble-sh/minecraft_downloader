# Minecraft Downloader

[![Build Status](https://travis-ci.org/marblenix/minecraft_downloader.svg?branch=master)](https://travis-ci.org/marblenix/minecraft_downloader)

A very simple application to download the latest version of the minecraft server .jar.

Downloads for Windows, Mac, and Linux can be found on the [releases](https://github.com/marblenix/minecraft_downloader/releases/latest) page.

```
minecraft_downloader 0.6.0
Download Minecraft's server.jar file

USAGE:
    minecraft_downloader [FLAGS] [OPTIONS]

FLAGS:
    -h, --help            Prints help information
    -n, --no-download     Do not download the latest .jar, instead output only the found version.
    -x, --no-overwrite    Do not overwrite the file on disk.
    -q, --quiet           Silence everything but errors and necessary output.

OPTIONS:
    -j, --jar-type <JAR_TYPE>    The type of jar to download. [env: MINECRAFT_JAR_TYPE]  [default: server]  [possible values: server, client]
    -v, --version <VERSION>      The specific version to download. Takes precedence over release_type. [env: MINECRAFT_VERSION]  [default: latest]
    -o, --output <OUTPUT>        Where to save the jar file. Overwrites the file if it already exists on disk. [env: MINECRAFT_OUTPUT]
    -t, --type <RELEASE_TYPE>    The type of release to download, only used if version is "latest". [env: MINECRAFT_RELEASE_TYPE]  [default: release]  [possible values: release, snapshot]
```

Examples:

```shell script
MINECRAFT_VERSION=17w49a MINECRAFT_OUTPUT=minecraft.jar ./minecraft_downloader_osx
```

```commandline
minecraft_downloader_windows.exe --version latest --type snapshot
```
