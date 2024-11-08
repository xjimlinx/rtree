// TODO: 统计文件数目
use colored::Colorize;
use std::env;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

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
fn print_tree(
    dir: &Path,
    prefix: String,
    depth: usize,
    max_depth: usize,
    show_hidden: bool,
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
    // filter从闭包获取该目录下的文件
    // 该闭包会对隐藏文件（即开头为'.'字符的文件）进行过滤
    // 当 show_hidden == true 时，
    // 条件全为真，也就是所有文件都会打印
    let mut entries = entries
        .filter_map(Result::ok)
        .filter(|e| show_hidden || !e.file_name().to_string_lossy().starts_with('.'))
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
                        // 0o111 = 0b001001001
                        // 即对应 rwx权限的 --x--x--x
                        if path.metadata()?.permissions().mode() & 0o111 != 0 {
                            "green"
                        } else {
                            "white"
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
            print_tree(&path, new_prefix, depth + 1, max_depth, show_hidden)?;
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
fn parse_args() -> (bool, usize, bool, Option<String>) {
    let args: Vec<String> = env::args().collect();
    let mut max_depth = 0;
    let mut show_hidden = false;
    let mut directory = None;
    let mut i = 1;

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
                        return (false, 0, false, None);
                    }
                } else {
                    println!(
                        "{}: missing value for {}",
                        "Tip".color("green").on_bright_white(),
                        args[i]
                    );
                    return (false, 0, false, None);
                }
            }
            "--all" | "-a" => {
                show_hidden = true;
            }
            "-v" | "--version" => {
                println!("rtree v{} © 2024 by {}", env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_AUTHORS"));
                return (false, 0, false, None);
            }
            "-h" | "--help" => {
                println!("Usage: rtree [OPTION] [DIRECTORY]");
                println!("       rtree [DIRECTORY] [OPTION]");
                println!();
                println!("Options:");
                println!("{:<30} {}", "-h, --help", "Print this help message");
                println!(
                    "{:<30} {}",
                    "--depth DEPTH, -L DEPTH", "Set the maximum depth of the tree (default: 0)"
                );
                println!(
                    "{:<30} {}",
                    "--all, -a", "Show hidden files and directories"
                );
                println!(
                    "{:<30} {}",
                    "[DIRECTORY]", "The directory to list (default: current directory)"
                );
                println!("{:<30} {}", "-v, --version", "Print the version of rtree");
                return (false, 0, false, None);
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
                        return (false, 0, false, None);
                    } else if !path.is_dir() {
                        println!(
                            "{}: '{}' is not a valid directory.",
                            "Tip".color("green").on_bright_white(),
                            path.display()
                        );
                        return (false, 0, false, None);
                    }
                }
            }
        }
        i += 1;
    }

    (true, max_depth, show_hidden, directory)
}

fn main() -> io::Result<()> {
    // 解析命令行参数
    // is_print_tree: 是否打印
    // max_depth: 打印层数，最小为1，默认全部（0）
    // show_hidden: 是否打印隐藏文件，默认为0,不打印，为1时打印
    let (is_print_tree, max_depth, show_hidden, directory) = parse_args();

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
    print_tree(&print_dir, String::new(), 1, max_depth, show_hidden)?;
    Ok(())
}
