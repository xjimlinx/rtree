// TODO: 统计文件数目
use colored::Colorize;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::io::ErrorKind;
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
#[cfg(target_family = "windows")]
use std::os::windows::fs::MetadataExt;
use std::path::Path;

struct TreePrintArgs {
    is_print_tree: bool,
    is_directory_only: bool,
    is_file_only: bool,
    max_depth: usize,
    show_hidden: bool,
    directory: Option<String>,
}

const EXIT_VAL: TreePrintArgs = TreePrintArgs {
    is_print_tree: false,
    max_depth: 0,
    show_hidden: false,
    directory: None,
    is_directory_only: false,
    is_file_only: false,
};
/// 文件类型

enum FileType {
    File,
    Directory,
    SymbolicLink,
    Unknown,
}

/// 打印文件和目录结构的递归函数
/// dir: 目录节点
/// prefix：打印缩进
/// depth：打印深度
/// max_depth：根节点打印深度
/// show_hidden：是否打印隐藏文件
/// is_directory_only：是否只打印目录
fn print_tree(
    dir: &Path,
    prefix: String,
    depth: usize,
    max_depth: usize,
    show_hidden: bool,
    is_directory_only: bool,
    is_file_only: bool,
) -> io::Result<()> {
    // 如果最大层数不等于0并且打印层数大于最大打印层数
    // （最大层数等于0的时候即不限制打印层数）
    // 直接返回
    if max_depth != 0 && depth > max_depth {
        return Ok(());
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            if e.kind() == ErrorKind::PermissionDenied {
                println!(
                    "{}: Permission denied: {}",
                    "Tip".color("green").on_bright_white(),
                    dir.display()
                );
            } else {
                println!(
                    "{}: Error reading directory: {}",
                    "Tip".color("green").on_bright_white(),
                    dir.display()
                );
            }
            return Ok(()); // 返回空，防止程序崩溃
        }
    };
    let mut entries = entries
        // filter_map 即 filter 和 map 的结合
        // 此处 filter_map 的作用是过滤掉错误的结果
        .filter_map(Result::ok)
        // 过滤器，只打印非隐藏文件，由变量 show_hidden 控制
        .filter(|e| show_hidden || !e.file_name().to_string_lossy().starts_with('.'))
        // 过滤器，只打印目录，由变量 is_directory_only 控制
        .filter(|e| {
            if is_directory_only {
                e.path().is_dir()
            } else {
                true
            }
        })
        // 过滤器，只打印文件，由变量 is_file_only 控制
        .filter(|e| {
            if is_file_only {
                e.path().is_file()
            } else {
                true
            }
        })
        .collect::<Vec<_>>();

    // 对文件项按字母顺序排序
    entries.sort_by(|a, b| {
        let a_bind = a.file_name();
        let a_name = a_bind.to_string_lossy();
        let b_bind = b.file_name();
        let b_name = b_bind.to_string_lossy();
        // 先按字母顺序排列
        let cmp = &a_name.to_lowercase().cmp(&b_name.to_lowercase());

        // 首字母相同，再按大小写顺序排序，小写优先
        if *cmp == std::cmp::Ordering::Equal {
            b_name.cmp(&a_name)
        } else {
            *cmp
        }
    });

    // 遍历上面获得的文件项，enumerate会返回一个元组
    // (index, value)
    // 即 (index, entry)
    // 直到打印到最后一个节点，会打印结尾连接线
    for (index, entry) in entries.iter().enumerate() {
        let path = entry.path();
        let is_last = index == entries.len() - 1;
        // let is_dir = entry.path().is_dir();
        let file_type = {
            if path.is_dir() {
                FileType::Directory
            } else if path.is_symlink() {
                FileType::SymbolicLink
            } else if path.is_file() {
                FileType::File
            } else {
                FileType::Unknown
            }
        };

        print!("{}", prefix);
        if is_last {
            print!("└── ");
        } else {
            print!("├── ");
        }

        println!(
            "{}",
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .color(match file_type {
                    FileType::Directory => "blue",
                    FileType::SymbolicLink => "bright cyan",
                    FileType::File => {
                        #[cfg(target_family = "unix")]
                        {
                            // 0o111 = 0b001001001
                            // 即对应 rwx权限的 --x--x--x
                            if path.metadata()?.permissions().mode() & 0o111 != 0 {
                                "green"
                            } else {
                                "white"
                            }
                        }
                        #[cfg(target_family = "windows")]
                        {
                            // 对于 Windows 系统，通过文件扩展名来判断文件是否可执行
                            let extension = path.extension().and_then(OsStr::to_str);
                            // 常见的可执行文件扩展名
                            let executable_extensions = ["exe", "bat", "cmd", "msi", "vbs", "ps1"];

                            if let Some(ext) = extension {
                                if executable_extensions.contains(&ext.to_lowercase().as_str()) {
                                    "green" // 文件是可执行的
                                } else {
                                    "white" // 文件不是可执行的
                                }
                            } else {
                                "white" // 文件不是可执行的
                            }
                        }
                    }
                    FileType::Unknown => "yellow",
                })
        );
        // 如果节点是一个目录，进入该节点继续打印
        // 如果这个目录是最后一个节点，那么新前缀前面不会有父目录的竖线
        // 否则，新前缀加一个竖线以及缩进
        // 然后开始打印
        if path.is_dir() {
            let new_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };
            print_tree(
                &path,
                new_prefix,
                depth + 1,
                max_depth,
                show_hidden,
                is_directory_only,
                is_file_only,
            )?;
        }
    }
    Ok(())
}

/// 解析命令行参数
/// 返回一个三元组
/// is_print_tree: 是否打印
/// max_depth: 打印层数，最小为1，默认全部（0）
/// show_hidden: 是否打印隐藏文件，默认为0,不打印，为1时打印
/// directory: 要打印的目录，如果没有提供则为 None
fn parse_args() -> TreePrintArgs {
    let args: Vec<String> = env::args().collect();
    let mut max_depth = 0;
    let mut show_hidden = false;
    let mut directory = None;
    let mut i = 1;
    let mut is_directory_only = false;
    let mut is_file_only = false;

    while i < args.len() {
        match args[i].as_str() {
            "--depth" | "-L" => {
                if i + 1 < args.len() {
                    if let Ok(d) = args[i + 1].parse() {
                        max_depth = d;
                        i += 1;
                    } else {
                        println!(
                            "{}: Invalid value for {}, expected a number.",
                            "Tip".color("green").on_bright_white(),
                            args[i]
                        );
                        return EXIT_VAL;
                    }
                } else {
                    println!(
                        "{}: missing value for {}",
                        "Tip".color("green").on_bright_white(),
                        args[i]
                    );
                    return EXIT_VAL;
                }
            }
            "--all" | "-a" => {
                show_hidden = true;
            }
            "-d" | "--directory" => {
                is_directory_only = true;
            }
            "--fileonly" => {
                is_file_only = true;
            }
            "-V" | "--version" => {
                println!(
                    "rtree v{} © 2024 by {}",
                    env!("CARGO_PKG_VERSION"),
                    env!("CARGO_PKG_AUTHORS")
                );
                return EXIT_VAL;
            }
            "-h" | "--help" => {
                print_help_msg();
                return EXIT_VAL;
            }
            _ => {
                // 如果都不是，表示这是要打印的目录
                if directory.is_none() {
                    directory = Some(args[i].clone());

                    // 如果提供的不是一个有效的路径，就返回
                    let path = Path::new(&args[i]);
                    if !path.exists() {
                        println!(
                            "{}: The specified directory '{}' does not exist.",
                            "Tip".color("green").on_bright_white(),
                            path.display()
                        );
                        return EXIT_VAL;
                    } else if !path.is_dir() {
                        println!(
                            "{}: '{}' is not a valid directory.",
                            "Tip".color("green").on_bright_white(),
                            path.display()
                        );
                        return EXIT_VAL;
                    }
                }
            }
        }
        i += 1;
    }
    if is_directory_only && is_file_only {
        println!(
            "{}: You can't specify both --directory and --fileonly.",
            "Tip".color("green").on_bright_white()
        );
        return EXIT_VAL;
    }
    let ret = TreePrintArgs {
        max_depth,
        show_hidden,
        directory,
        is_directory_only,
        is_file_only,
        is_print_tree: true,
    };
    ret
}

fn print_help_msg() {
    println!("Usage: rtree [OPTION] [DIRECTORY]");
    println!("       rtree [DIRECTORY] [OPTION]");
    println!();
    println!("Options:");
    println!("{:<30} {}", "-h, --help", "Print this help message");
    println!(
        "{:<30} {}",
        "[DIRECTORY]", "The directory to list (default: current directory)"
    );
    println!(
        "{:<30} {}",
        "--depth DEPTH, -L DEPTH", "Set the maximum depth of the tree (default: 0)"
    );
    println!(
        "{:<30} {}",
        "--all, -a", "Show hidden files and directories"
    );
    println!("{:<30} {}", "-d, --directory", "List directories only");
    println!("{:<30} {}", "--fileonly", "List files only");
    println!("{:<30} {}", "-V, --version", "Print the version of rtree");
}

fn main() -> io::Result<()> {
    // 解析命令行参数
    // is_print_tree: 是否打印
    // max_depth: 打印层数，最小为1，默认全部（0）
    // show_hidden: 是否打印隐藏文件，默认为0,不打印，为1时打印
    // directory: 要打印的目录，如果没有提供则为 None
    // is_directory_only: 仅列出目录
    // is_file_only: 仅列出文件
    let ret_args = parse_args();
    let (is_print_tree, max_depth, show_hidden, directory, is_directory_only, is_file_only) = (
        ret_args.is_print_tree,
        ret_args.max_depth,
        ret_args.show_hidden,
        ret_args.directory,
        ret_args.is_directory_only,
        ret_args.is_file_only,
    );

    // 如果不打印，直接返回
    if is_print_tree == false {
        return Ok(());
    }
    // 获取当前目录
    let print_dir = match directory {
        Some(dir) => Path::new(&dir).to_path_buf(),
        None => env::current_dir()?,
    };

    // 如果要打印的是当前目录
    if print_dir == env::current_dir()? {
        println!("{}", ".".color("blue"));
    } else {
        println!("{}", print_dir.display().to_string().color("blue"));
    }
    // println!("{}", current_dir.display());
    // 递归打印目录下文件
    print_tree(
        &print_dir,
        String::new(),
        1,
        max_depth,
        show_hidden,
        is_directory_only,
        is_file_only,
    )?;
    Ok(())
}
