use colored::Colorize;
use std::env;
use std::fs;
use std::io;
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

    // 从闭包获取该目录下的文件
    // 该闭包会对隐藏文件（即开头为'.'字符的文件进行过滤）
    // 当 show_hidden == true 时，
    // 条件全为真，也就是所有文件都会打印
    let mut entries = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|e| show_hidden || !e.file_name().to_string_lossy().starts_with('.'))
        .collect::<Vec<_>>();

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
        // 如果这个目录是最后一个节点，那么新前缀不加长
        // 否则，新前缀加一个竖线
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
fn parse_args() -> (bool, usize, bool) {
    let args: Vec<String> = env::args().collect();
    let mut max_depth = 0;
    let mut show_hidden = false;

    for i in 1..args.len() {
        match args[i].as_str() {
            "--depth" | "-L" => {
                if i + 1 < args.len() {
                    if let Ok(d) = args[i + 1].parse() {
                        max_depth = d;
                    }
                } else {
                    eprintln!("ERROR: unexpected argument")
                }
            }
            "--all" | "-a" => {
                show_hidden = true;
            }
            "-h" | "--help" => {
                println!("Usage: tree [OPTION] [DIRECTORY]");
                println!("Options:");
                println!("{:<30} {}", "-h, --help", "Print this help message");
                println!("{:<30} {}", "--depth DEPTH, -L DEPTH", "Set the maximum depth of the tree (default: 0)");
                println!("{:<30} {}", "--all, -a", "Show hidden files and directories");
                return (false, 0, false);
            }
            _ => {
            }
        }
    }

    (true, max_depth, show_hidden)
}

fn main() -> io::Result<()> {
    // 解析命令行参数
    // is_print_tree: 是否打印
    // max_depth: 打印层数，最小为1，默认全部（0）
    // show_hidden: 是否打印隐藏文件，默认为0,不打印，为1时打印
    let (is_print_tree, max_depth, show_hidden) = parse_args();

    if is_print_tree == false {
        return Ok(());
    }
    // 获取当前目录
    let current_dir = env::current_dir()?;
    // 打印当前目录
    println!("{}", ".".color("blue"));
    // println!("{}", current_dir.display());
    // 递归打印目录下文件
    print_tree(&current_dir, String::new(), 1, max_depth, show_hidden)?;
    Ok(())
}
