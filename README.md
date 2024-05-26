# jsdu

JSON file size analyzer. Like ncdu but for JSON files.

Ever tried to open a big JSON file with your favorite text editor, only for it to start lagging?
Maybe it opened fine but it's minified and you don't know what are you looking at?

Worry not, jsdu will help you make sense of it.

## Install

```
cargo install jsdu
```

## Usage

Interactive mode is not implemented yet, but you can do basic stuff using the command line:

```
# Open JSON file in interactive mode (not implemented yet)
jsdu bigFile.json
# Minify/prettify files (in place)
jsdu min bigFile.json
jsdu fmt bigFile.json
# Explore size of JSON structure
jsdu show bigFile.json
# JSON Pointers are supported (RFC 6901)
jsdu show bigFile.json --pointer "/data/0/"
```
