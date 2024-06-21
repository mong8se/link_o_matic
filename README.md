# link_o_matic

## Learning Rust on my own

### just one more dot file symlinker_o_matic

#### uses convention over configuration principle

Expects an environment variable LINKOMATIC_ROOT which points to where
dotfiles are checked out. Doesn't have to always be set, just need to
set it when you run link_o_matic

Expects an environment variable LINKOMATIC_HOSTNAME which is whatever
string you want to represent the host you are on. Can be the same as
HOSTNAME or different if you want to obfuscate the files for that host
in your repo.

Any files that begin with _${LINKOMATIC_HOSTNAME} will only be
symlinked. Other machines should get their own dang LINKOMATIC_HOSTNAME.
When symlinked it will be renamed _machine so you can just source that
file without knowing which hostname.

Any files that begin with _mac or _linux will only be symlinked on that
host. When symlinked it will be renamed _platform so you can just source
that file without knowing which platform.


## Commands

install cleanup sync autocleanup implode

### install

Starts in `LINKOMATIC_ROOT/home`

If it is a dir, it is recursed.
If it is a file, a symlink is made to it.
If it is a symlink, a symlink is made to its target.

### cleanup

Looks for links in ~ that link to something in `LINKOMATIC_ROOT/home`
Look in the root of ~, or recursively through any ~/dir that has a
matching LINKOMATIC_ROOT/home/dir to avoid scanning through the entire
home directory. So if you remove a top level directory you'll leave
orphaned links at this point, unless you remove them from the repo first
and cleanup before removing the top level directory.
TODO: is there a better way?

If it's a broken or invalid link, prompts to delete.

### sync

First `install` then `cleanup`.

### autocleanup

Same as `cleanup` but doesn't prompt.

### implode

Looks for links in ~ that link to something in `LINKOMATIC_ROOT/home`
Prompts to delete.

## Aliases (symlinks to spawn symlinks)

If you create a symlink inside `LINKOMATIC_ROOT/home` that is a valid _relative_ path
to a file inside `LINKOMATIC_ROOT`, when the link is created it will point
directly to the target of the symlink, not the intermediary symlink.

For example start in your dot files:

    cd ~/$LINKOMATIC_ROOT

This step isn't strictly necessary but will allow tab completion of your
target

    cd home

Now make a relative link to your target file (you'd link to `home/vimrc` if
you skipped above step)

    ln -s ../config/nvim/init.vim vimrc

Now you will have a symlink in `LINKOMATIC_ROOT/home/vimrc` that points to
`LINKOMATIC_ROOT/config/nvim/init.vim` but relatively.

When you run install your `~/.vimrc` will point to your
`LINKOMATIC_ROOT/config/nvim/init.vim` ... not to the symlink `LINKOMATIC_ROOT/home/vimrc`
