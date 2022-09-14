# Intel Alderlake E-Core Detection

Determine if you have a 12th-gen Intel CPU with a hybrid core.

## Description
Determine the topology of a 12th-gen Intel CPU (Alder Lake) in regard to its
Intel Atom (E-Core) and Intel Core (P-Core) count and index. 

There is a library contains all functionality with sync & async variants.

The program executable must be the first argument passed to the program. (99% it will be by default.)

## Getting Started

### Dependencies

* [Taskset](https://man7.org/linux/man-pages/man1/taskset.1.html#:~:text=The%20taskset%20command%20is%20used,of%20CPUs%20on%20the%20system.)
\- is required to target specific CPU cores.

### Installing
No binaries are provided.

Clone the repo and run make install. 
```zsh
make install_local
```

### Flags

```
-s or --single (DEFAULT) // Determine if the core the program is executed on is P or E

-a --all // Scrape all CPU's to get the total and partition of CPU cores.
```

## Help

Any advise for common problems or issues.
```
-h or --help // Explain the flags and their usage.
```


## Version History

* 0.1
    * Initial Release

## License

This project is licensed under the MIT License - see the LICENSE.md file for details
