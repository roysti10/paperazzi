<h1 align=center>
Paperazzi
</h1>

<p align=center>
A simple TUI to search Research Papers
</p>

<p align=center>
<a href="https://crates.io/crates/paperazzi"><img alt="Crates.io badge" src="https://img.shields.io/crates/v/paperazzi"></a>
</p>

# Usage

## Install

### Download from Github

Go to [Releases](https://github.com/lucasace/paperazzi/releases/latest) and download the latest binary/executable

For Linux the binary is `paperazzi`

For windows the executable is `paperazzi.exe`

and Run as

`./paperazzi` in Linux

`./paperazzi.exe` in Windows

### From Cargo

```
cargo install paperazzi
```

### Build Manually

```
cargo build

```

For Linux the binary is `target/debug/paperazzi`

For Windows the exe is `target/debug/paperazzi.exe`

## Run Paperazzi

```
paperazzi <your query>
```

Example

```
paperazzi "GAN"
```

## Download a paper

```
paperazzi -d <doi url>
```

Example 

```
paperazzi -d https://doi.org/10.1109/ICCV.2017.405
```

## Options

See `paperazzi --help` for a list of options

# License

Apache License 2.0
