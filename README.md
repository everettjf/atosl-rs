# atosl-rs
atosl by rust

> Tested on dwarf and macho

# install

cargo install atosl


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
