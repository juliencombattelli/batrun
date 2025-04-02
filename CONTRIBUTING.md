# How to contribute

## Submitting changes

Please send a [GitHub Pull Request to batrun](https://github.com/juliencombattelli/batrun/pull/new/main)
with a clear list of what is done (read more about [pull requests](http://help.github.com/pull-requests/)).
When you send a pull request, please update the internal validation test suite
accordingly. Please follow the coding conventions below and make sure all of
your commits are atomic (one feature per commit) to make the review easier.

Always write a clear log message for your commits. One-line messages are fine
for small changes, but bigger changes should add some details in the commit
description.

## Coding conventions

### Minimum Bash version

Batrun is written in Bash and required at least version 4.4.

### Shell options

The main script is executed with a strict set of options:
  * -e (-o errexit): Exit immediately if a command fails
  * -u (-o noundef): No variable expansion is allowed on undefined variables
  * -o pipefail: If any command in a pipeline fails, the pipeline will return with an error

While those options makes error handling sometimes more cumbersome, they help to catch errors that
would otherwise be silently ignored.
Refer to https://mywiki.wooledge.org/BashFAQ/105 for some `errexit` pitfalls.

If a command is allowed to fail, append `|| true` and comment why the error might be expected.

If a command needs a more relaxed execution environment, those options can be disabled for that command.
You have three way to do this (from the preferred and most safe to the most risky):
  * call your command from a dedicated subprocess, eg. `bash -c "command"`
  * call your command from a dedicated function/subshell that uses `local -; set +<option>`
    to disable some options locally and automatically restore the original ones when the function returns
  * manually backup/restore the options (not desirable)

### Shellcheck

Use Shellcheck to capture common bash usage errors, eg. unquoted variable
expansion may cause unwanted word splitting.

Common code smells that Shellcheck can spot will not be have a dedicated section
in this document.

### Variable declaration

TODO
declare/local, readonly, array, etc
Prefer readonly instead of declare -r for better readability

### Function definition and  parameter passing

Use `function fn_name {}` to define a function. Do not use the empty `()`.

Always associate a name to a parameter:
```bash
function fn_name {
    local [-r] PARAM_NAME="$1"
}
```

To pass complex data like arrays to a function:
TODO readarray

### Command options

Long options must be used instead of short ones for readability.
