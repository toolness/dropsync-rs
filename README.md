This is a command-line tool I, Atul Varma, wrote to help me synchronize
application-specific data (usually saved game data) across multiple
computers using Dropbox.

Note that this tool doesn't actually use the network itself: it simply
assumes that the directory `Dropbox`, located off the user's home directory,
is automatically synchronized with other computers. It could be a shared
network drive, or it could even be manually copied to other computers via
USB stick.

**Disclaimer:** I wrote this tool because I wanted to write something in
Rust. It's likely that there are better alternatives out there!

## Installation

You'll need [Rust](https://www.rust-lang.org/).

Clone this repository, enter it in your terminal, and run:

```
cargo install --path . --force
```

Now the `dropsync` executable should be on your path. You can verify
this by running:

```
dropsync --help
```

Before actually using the tool, however, you'll need to configure it.

## Configuration

Before using this tool, create a file called `dropsync.toml` in the root of
your Dropbox folder.  It should contain entries for each app you want to
synchronize like so:

```toml
[MyFunkyGame]
path = "C:\\Users\\Atul\\AppData\\Local\\MyFunkyGame\\Saved\\SaveGames"
dropbox_path = "Games/MyFunkyGame"

[MyOtherGame]
path = "C:\\Users\\Atul\\Documents\\My Games\\MyFunkyGame"
dropbox_path = "Games/MyFunkyGame"
```

Each section corresponds to a specific application whose data you want to
synchronize and has the following entries:

* `path` is the absolute path to where the application expects to find
  and save its data.
* `dropbox_path` is the path relative to the Dropbox folder where the
  data will be synchronized.
* `disabled` is an optional boolean; if `true`, the application
  entry will be ignored.
* `include_only` is a glob pattern, like `*.sv`, which makes dropsync
  only synchronize files that match the pattern.
* `play_path` is the optional path to where the
  application's executable is. If provided, you will be able to
  use the `dropsync play <app name>` command, which may be
  convenient.
* `play_root_path` is an optional absolute path to an ancestor
  directory of the application's executable. If supplied, `play_path`
  will be appended to it (otherwise, `play_path` should be absolute).

  Most significantly, this path will also be used as the root directory
  to watch to determine whether the app has finished running. If,
  for instance, the `play_path` points to a "launcher" that launches
  the actual app, which is also under `play_root_path`, then providing
  this value will ensure that dropsync doesn't try to synchronize
  files until after the actual app has finished running.

If different computers have the applications at different locations, a
separate subsection denoted by the computer's hostname can store
host-specific configuration overrides, e.g.:

```toml
[MyFunkyGame]
# The default app path on all computers unless overridden.
path = "C:\\Users\\Atul\\AppData\\Local\\MyFunkyGame\\Saved\\SaveGames"
dropbox_path = "Games/MyFunkyGame"

[MyFunkyGame.MY_WEIRD_DESKTOP_COMPUTER]
# This will override the default app path on MY_WEIRD_DESKTOP_COMPUTER.
path = "F:\\MyFunkyGame\\Saved\\SaveGames"
```

Note that in the above example `MY_WEIRD_COMPUTER` is the name of the
computer. (On Windows, the name of your computer can be found by
typing "computer name" into the search box at the bottom-left of the
task bar; on other systems, try typing `hostname` in the terminal.)

Note that all directories do need to exist before running the program,
so you'll want to create them manually if they don't already exist.

## Usage

Once you've created your `dropsync.toml`, you can synchronize your
application data by running:

```
dropsync
```

The synchronization process is imperfect but should work in most
cases, with the following assumptions:

* You remember to manually run this tool before and after you're
  done using the relevant applications. Alternatively, you can
  also use `dropsync play <app name>` to first synchronize,
  then play (assuming `play_path` has been set for the relevant
  app), and then re-synchronize the app.
* No one else is using the applications at the same time as you
  on your other computers.

The tool works by comparing the contents of the application's data
folder and its Dropbox analog.  If one folder is non-empty, has
no files that are older than their equivalent in the other folder,
and has at least one file that is _newer_ than its equivalent in
the other folder, then the folder is considered to be the newer
version of the data, and its entire contents are copied to the
other folder. Files in the older folder that don't exist in
the newer folder are deleted.

If the contents of both folders aren't exactly equal, and if
neither is judged to be newer than the other, then the user
is prompted to manually resolve the conflict.

## Version history

### 1.0.0

Initial release.

### 1.1.0 - 2020-10-10

* Applications are now synced in alphabetical order.
* Added support for the `disabled` field to `dropsync.toml`.
