# conman

**Work in progress**

Conman is a CLI tool that solves the issue of storing app configuration files in
various directories (typically, they live in various code projects' folders)
making it difficult to move these files to another dev machine. Such
configuration files are often excluded from version control systems due to
secrets that they may contain. Conman manages those files and stores them in one
common location making it easy to transfer these files between dev machines
(potentially using private version control repository that could store all these
config files).

For the .NET developers, these could be `appsettings.json` files. For the NodeJS
developers, these could be `.env` files. Often such files contain secrets that you
want to keep to yourself.

The tool is the most useful for those who work on more than one computer with the 
same projects. It's also helpful if you like to change your operating system often.

## Configs location

The files managed by Conman are stored in the following locations, depending on
the operating system:

- macOS: `/Users/Alice/Library/Application Support/com.marcinjahn.conman/configs`
- Linux: `/home/alice/.config/conman/configs`
- Windows: `C:\Users\Alice\AppData\Roaming\marcinjahn\conman\configs`

Conman would be storing your projects' configs in folders stored in the
`configs` directory. Each project would have its own separate folder in the
`configs` directory.

Moving to another machine is more straightforward since all your app settings
files (at least the ones you manage with Conman) are stored in one central
location that you can transfer to the new machine.

## Usage

The following is the typical usage of conman:

1. You have some code project, let's say under `/home/user/code/app1`.
2. You can enable management of config files in that project by Conman, as
   follows:

```sh
cd /home/user/code/app1
conman init # you'll be asked to provide a name for a new project
```

At this point, Conman knows about that project, and it is able to manage config
files for it.

3. Let's add some config file to our project

```sh
conman add appsettings.json # optionally add '-g' flag to add the file to .gitignore
```

If the `appsettings.json` file is already present in the directory, the command
above would turn it into a Conman-managed file. If the file does not exist, it
would be created as a Conman-managed file.

The `appsettings.json` file becomes a symlink pointing to a location managed by
Conman.

All the Conman-managed projects have their config files stored in the central
directory making it easy to transfer the files to another dev machine if you
need to do so.

### Transfering configs to another machine

To move the config files to another machine, copy the `configs`
[directory](#configs-location) and put it in the appropriate
[location](#configs-location) (depending on your operating system) on the target
machine.

**Do not** copy the `config.json` file that can be found alongside the `configs`
directory. This file is normally unique per machine. It makes sense to copy it
(and place it alongside the `configs` directory on the target machine) only if
all of your Conman-managed projects are in the exact same locations on the
target machine as they are on the source machine. Otherwise, just leave the
`config.json` in the source machine.

> It would be nice to have commands such as `conman export/import` to make it
> easier to move the configs. It could be added in the future (also by you, in a PR ;))

When the `configs` directory is there (and Conman is installed) on the new
machine, you can initialize your projects, just like you initially did it on the
first machine. This time, however, Conman will "see" that there are some
unassociated configs available, and for any `conman init` command, that you
invoke, you would be asked if you want to associate one of the configs you copied
over with the project that you're initializing.

Here's an example of that:

```sh
## On a new machine
git clone some-repo/app1
cd app1
conman init # you'll be asked if you want to create a fresh project or associate it with one of the existing configs
```

Once you initialize the project on the new machine, Conman will bring in all the
config files of that project.

Note that you can use `conman --help` to learn how to use Conman. Each
subcommand has its own `--help` (e.g. `conman add --help`).

## Limitations

### Project subdirectories are not supported

Conman can manage only one-level depth of files in a project. For example, if
you initialize the following path as a Conman project - `/home/user/code/app1` - you can
`conman add` only files that are directly in that directory. 

If you see a use case for lifting up that limitation, feel free to create a Pull
Request or create an issue for it.


## Development

You can install the app locally using:

```sh
git clone https://github.com/marcinjahn/conman
cd marcinjahn/conman
cargo install --path .
```

For now, this is the only way to install Conman. Soon it will also be available
on crate.io and potentially some other package management systems (such as
brew).