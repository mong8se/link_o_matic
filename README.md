# link_o_matic

## Learning Rust on my own

### just one more dot file symlinker_o_matic

#### uses convention over configuration principle

Expects an environment variable REPO_LOCATION which points to where
dotfiles are checked out. Doesn't have to always be set, just need to
set it when you run link_o_matic

Expects an environment variable HOST42 which is whatever string you want
to represent the host you are on. Can be the same as HOST or different
if you want to obfuscate the files for that host in your repo.

Any files that begin with _${HOST42} will only be symlinked. Other
machines should get their own dang HOST42. When symlinked it will be
renamed _machine so you can just source that file without knowing the
ultimate HOST 42

Any files that begin with _mac or _linux will only be symlinked on that
host. When symlinked it will be renamed _platform so you can just source
that file without knowing which platform.


## Commands

install cleanup autocleanup implode

### install

Starts in `REPO_LOCATION/home`

If it is a dir, it is recursed.
If it is a file, a symlink is made to it.
If it is a symlink, a symlink is made to its target.

### cleanup

Looks for links in ~ that link to something in `REPO_LOCATION/home`
If it's a broken or invalid link, prompts to delete.

### autocleanup

Same as above but doesn't prompt.

### implode

Looks for links in ~ that link to something in `REPO_LOCATION/home`
Prompts to delete.
