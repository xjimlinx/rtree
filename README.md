# rtree

> 一个用Rust编写的tree工具的简易版本

使用方法：

```bash
Usage: rtree [OPTION] [DIRECTORY]
       rtree [DIRECTORY] [OPTION]

Options:
-h, --help                     Print this help message
[DIRECTORY]                    The directory to list (default: current directory)
--depth DEPTH, -L DEPTH        Set the maximum depth of the tree (default: 0)
--all, -a                      Show hidden files and directories
-d, --directory                List directories only
--fileonly                     List files only
-V, --version                  Print the version of rtree
```

编译：

```bash
make build 
```

注意`MODE=debug`可作为调试，但是占用空间较大

请使用`MODE=release`用于安装

测试（测试用例现在还没写）：

```bash
make test
# TODO:
```

安装（将安装到 `/usr/local/bin 目录下`）：

```bash
make install MODE=release
```
