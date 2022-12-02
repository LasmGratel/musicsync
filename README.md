# musicsync

Convert your music folder into another format, for portable purpose.

I write this tool to sync my music folder to portable devices (two laptops, one cell phone), because I need to compress my lossless music to lossy format for portable purpose.

Default options are **Opus 192K**, this is pretty much transparent and playable on most modern devices.

## Usage

**Requires ffmpeg executable in PATH.**

You can simply ignore those whole lot shitty options. Specify your music folder and a target folder, and that's it. Music will be converted into opus format and covers will be copied.

```
musicsync [OPTIONS] <INPUT> <OUTPUT> [EXTENSION]

Arguments:
  <INPUT>      Input directory for walking
  <OUTPUT>     Output directory
  [EXTENSION]  Output file extension, default to opus

Options:
  -f, --force                 Force overwrite existing files
      --preserve              Preserve target folder files, even if they don't exist in source dir
      --dontcopycover         Don't copy cover images
      --ffmpeg <FFMPEG_PATH>  Specify ffmpeg program path
  -o, --options <OPTIONS>     Options to be passed to ffmpeg, default to -c:a libopus -b:a 192K -vbr on -cutoff 0
  -t, --filetype <TYPES>      Specify file types to be converted, split with a ',' character. Default to mp3,aac,aif,flac,ogg,wav
      --cover <COVER>         Cover image suffix (case-insensitive). Default to Cover.jpg,Cover.png,AlbumArtSmall.jpg,AlbumArtwork.png
  -h, --help                  Print help information
  -V, --version               Print version information
```

## TODO

- [ ] Presets, for mp3, aac, customizable.
- [ ] Bundle ffmpeg?
- [ ] An working example for Syncthing + this
