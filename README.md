# Minecraft Downloader

[![Build Status](https://travis-ci.org/marblenix/minecraft_downloader.svg?branch=master)](https://travis-ci.org/marblenix/minecraft_downloader)

A very simple application to download the latest version of the minecraft server .jar.

Downloads for Windows, Mac, and Linux can be found on the [releases](https://github.com/marblenix/minecraft_downloader/releases/latest) page.

```
Minecraft Downloader
Download Minecraft's server.jar file

USAGE:
    minecraft_downloader [FLAGS] [OPTIONS]

FLAGS:
    -h, --help           Prints help information
    -n, --no-download    do not download the latest .jar, instead output only the found version

OPTIONS:
    -v, --version <MINECRAFT_VERSION>    the specific version to download [env: MINECRAFT_VERSION=]  [default: latest]
    -o, --output <OUTPUT>                where to save the jar file [env: MINECRAFT_OUTPUT=]  [default: latest]
    -t, --type <TYPE>                    the type of release to download, only used if version is "latest" [env: MINECRAFT_RELEASE_TYPE=]  [default: release]  [possible values: release, snapshot]
```

Examples:

```shell script
MINECRAFT_VERSION=17w49a MINECRAFT_OUTPUT=minecraft.jar ./minecraft_downloader_osx
```

```commandline
minecraft_downloader_windows.exe --version latest --type snapshot
```
