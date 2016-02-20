# meku [![Build Status](https://travis-ci.org/KevinBalz/meku.svg?branch=master)](https://travis-ci.org/KevinBalz/meku)

meku is a simple content pipeline for game makers

# What it does

In the simplest case, it just copies your content folder. But you can provide rules for specific extensions, which will process those files.

Some use cases could be:
* Exporting .psd to a image format your engine supports (e.g. .png)
* Export your Tiled .tmx files to lua, json ...
* Precompile your script files

and many others.

So for example your content looks like this.
```
content/
├── main.lua
├── meku.yml
├── sprite.psd
└── map.tmx
```
After processing with meku:
```
.../content/
├── main.lua
├── sprite.png
└── map.lua
```

Note that the `meku.yml` is missing in the resulting folder.
In this yaml the rules are specified and is skipped in the process.
The `sprite.psd` and `map.tmx` got replaced by `sprite.png` and `map.lua`.

The `meku.yml` could have been looked like this:
```yaml
"*.psd":
  commands:
    - convert %{src_file} %{tar_dir}/%{src_file_stem}.png
"*.tmx":
  commands:
    - tiled --export-map \"Lua files (*.lua)\" %{src_file} %{tar_dir}/%{src_file_stem}.lua
```

It describes the rules convert .psd files and .tmx files to their useable end result.


# Usage

# Goals

# TODO

- [ ] Github Releases
- [ ] Flesh out the meku.yml format
- [ ] Proper Readme

# Note

meku is still in heavy development.

*Licensed under MIT*
