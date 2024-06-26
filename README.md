# link_o_matic

## Learning Rust on my own

### just one more dot file symlinker_o_matic

NOTE THIS IS BETA AND ONLY USED BY ME SO MAYBE DON'T USE THE DELETE
FUNCTIONS UNTIL YOU ARE SURE I KNOW WHAT I AM DOING -- But I've been
using it without accidentally deleting files so far.

#### uses convention over configuration principle:

Expects an environment variable LINKOMATIC_ROOT which points to where
dotfiles are checked out. Doesn't have to be exported, just need to
set it when you run link_o_matic

Inside your LINKOMATIC_ROOT you'll make a directory called `home`
which will be the files/dirs you'll want linked without the
leading `.` in the names
Examples:
  * `home/thing` will get linked to `~/.thing`
  * `home/config/other_thing` will get linked to `~/.config/other_thing`

Expects an environment variable LINKOMATIC_HOSTNAME which is whatever
string you want to represent the host you are on. Can be the same as
HOSTNAME or different if you want to obfuscate your host names in your
git repo.

Any files that begin with an underscore will only be symlinked IF:
  * They start with `_${LINKOMATIC_HOSTNAME}`
    * Will be symlinked as `_machine` and the rest
      * For example `_bob.nvim.lua` will get linked as `_machine.nvim.lua` only on host bob
    * Other machines should get their own dang LINKOMATIC_HOSTNAME
  * They start with one of `_mac` `_linux` and your OS is mac or linux
    * Will be symlinked as `_platform`
      * For example `_mac.fish` will get linked as `_platform.fish` only on a mac

This way your config files can reference your `_machine` and `_platform`
files and not have to know the actual names.

## Commands

`install cleanup sync autocleanup implode`

### install

Starts in `LINKOMATIC_ROOT/home`

If it is a dir, it is recursed.
If it is a file, a symlink is made to it.
If it is a symlink, a symlink is made to its target.
Its name will have a `.` prepended

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
