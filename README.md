# conman

**Work in progress**

Conman is a CLI tool that solves the issue of storing app configuration files in
various directories (typically, they live in various app project folders) making
it difficult to move these files to another dev machine. Such configuration
files are often excluded from version control systems due to secrets that they
may contain. Conman manages those files and stores them in one common location
making it easy to transfer these files between dev machines (potentially using
private version control repository that could store all these config files).

The configuration files are accessible to their projects via soft links that
conman creates.

## Development

You can install the app locally using:

```sh
git clone https://github.com/marcinjahn/conman
cd marcinjahn/conman
cargo install --path .
```

Use `conman --help` to learn how to use it.