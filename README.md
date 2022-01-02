# atosl-rs

🦀️atos for linux by rust - A partial replacement for Apple's atos tool for converting addresses within a binary file to symbols.


> Tested on dwarf and macho

# install

1. install rust via : https://www.rust-lang.org/tools/install
2. cargo install atosl


# usage

```
USAGE:
    atosl [OPTIONS] -o <OBJECT_PATH> -l <LOAD_ADDRESS> [ADDRESSES]...

ARGS:
    <ADDRESSES>...    Addresses need to translate

OPTIONS:
    -l <LOAD_ADDRESS>        Load address of binary image
    -o <OBJECT_PATH>         Symbol file path or binary file path

```


# sample 

```
// for dwarf
atosl -l 4581015552 -o "full path to dwarf file" 4674962060 4786995348

// for macho
atosl -l 9093120 -o "full path to libsystem_malloc.dylib" 6754325196 
```

# optimize

feel free to make a pull request :)