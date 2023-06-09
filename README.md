# Overview
This Rust binary cleans up the currrent directory of duplicate files. It scans for files that have been
downloaded and given similar names, and then it generates a bash script to clean up the duplicates.

The idea is that the code can be run once without deletions or renames, piping the script to a file. Once
the file has been examined for unwanted changes, the script can be run from bash to remove unneeded files.

The code emits blocks of bash comments that support the reasoning of the changes. This makes it easier
to read the proposed changes, to understand their effects, and to develop trust in the operation of the code.

# Usage
For initial usage:
``` bash
file-dup --filetype=".zip" > remove-dups.sh
# And later:
sh remove-dups.sh
```

After developing trust in the code, you may run it like this:
``` bash
file-dup --filetype=".zip" | sh
```

# Command line arguments
The code has a single non-admin command line argument: `--filetype`. The expectation is that argument begins with a `.`.

# Help
The help looks like this:
``` bash
File deduplicator

Usage: file-dup [OPTIONS]

Options:
  -f, --filetype <filetype>  [default: .pdf]
  -h, --help                 Print help
  -V, --version              Print version
```
