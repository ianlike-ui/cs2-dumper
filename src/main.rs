// ============================================================================
// main.rs —— cs2-dumper 的入口（程序做什么、怎么跑）
//
// ----------------------------------------------------------------------------
// 🎓 概念卡 ㉔：Cargo、crate、mod、use（Rust 的工程组织方式）
// ----------------------------------------------------------------------------
// 【是什么】
// - Cargo 是 Rust 的"构建工具 + 包管理器"（类似 Maven/npm）：Cargo.toml
//   声明依赖和配置，`cargo build` 一键编译。
// - crate（箱）= 一个可编译单元（一个项目 = 一个 crate）。
// - mod = 模块：把一个 crate 拆成小块。`mod analysis;` 表示"当前目录下
//   有 analysis 模块（analysis/mod.rs 或 analysis.rs）"。
// - use = 引入：`use memflow::prelude::v1::*;` 把别人的名字引入当前作用域，
//   类似 C++ 的 using。
// 【最小示例】
//   // main.rs
//   mod math;                    // 声明子模块（同目录有 math.rs）
//   fn main() { let s = math::add(1, 2); }
//
//   // math.rs
//   pub fn add(a: i32, b: i32) -> i32 { a + b }   // pub = 对外可见
//
// 【自己动手要点】
// 1) `cargo new xxx` 建项目，`cargo run` 跑，`cargo add 包名` 加依赖；
// 2) 文件布局有约定：模块名 就是文件名；目录要有 mod.rs（旧式）或用
//    同名文件 + 子目录（新式，本项目用的是 mod.rs 风格）；
// 3) 引用别人：`crate::analysis::offsets` = "本项目 analysis 模块里的 offsets"。
// ----------------------------------------------------------------------------
//
// 【这个程序是干什么的（用大白话讲）】
// CS2 游戏的数据（玩家血量、坐标、敌人列表……）都存在游戏进程的内存里。
// 但"这些数据在内存的哪个位置"每次游戏更新都会变。cs2-dumper 的任务就是：
// 读游戏进程内存，把"各个数据的位置表"（偏移量）挖出来，生成
// C++/C#/Rust 等语言的头文件，供其他程序（比如 cs2_cheat）编译时使用。
//
// 【流程】
//   1. 解析命令行参数（用 clap 库）；
//   2. 用 memflow 打开 CS2 进程（默认本机直接读，也可以用 pcileech 等
//      远程内存读取器从另一台机器/虚拟机读）；
//   3. analysis::analyze_all：执行四类分析（按键、接口、偏移、Schema 类表）；
//   4. output::Output：把分析结果写成 5 种语言的文件（cs/hpp/json/rs/zig）。
//
// 【概念速览】mod / use / Result 与 ? 等 Rust 基础，见上方 🎓 概念卡 ㉔㉕
// （完整索引见《搭建路线图与概念索引.md》）。
// ============================================================================

#![allow(dead_code)]
#![allow(unused_imports)]

use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

use anyhow::Result;

use clap::{ArgAction, Parser};

use log::{LevelFilter, info};

use memflow::prelude::v1::*;

use simplelog::*;

use output::Output;

mod analysis;
mod memory;
mod output;
mod source2;

#[derive(Debug, Parser)]
#[command(author, version)]
struct Args {
    /// The name of the memflow connector to use.
    #[arg(short, long)]
    connector: Option<String>,

    /// Additional arguments to pass to the memflow connector.
    #[arg(short = 'a', long)]
    connector_args: Option<String>,

    /// The types of files to generate.
    #[arg(
        short,
        long,
        value_delimiter = ',',
        default_values = ["cs", "hpp", "json", "rs", "zig"]
    )]
    file_types: Vec<String>,

    /// The number of spaces to use per indentation level.
    #[arg(short, long, default_value_t = 4)]
    indent_size: usize,

    /// The output directory to write the generated files to.
    #[arg(short, long, default_value = "output")]
    output: PathBuf,

    /// The name of the game process.
    #[arg(short, long, default_value = "cs2.exe")]
    process_name: String,

    /// Increase logging verbosity. Can be specified multiple times.
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,

    /// Prevent creation of the cs2-dumper.log file.
    #[arg(short, long)]
    no_log_file: bool,
}

// ============================================================================
// main 函数：程序的入口。Rust 程序从这里开始执行。
// 返回 Result<()>：成功返回 Ok(())，出错返回 Err（会被打印出来并以非 0 退出码结束）。
// ----------------------------------------------------------------------------
// 🎓 概念卡 ㉕：Result 与 ?（Rust 的错误处理）
// ----------------------------------------------------------------------------
// 【是什么】Rust 没有 C++ 的 try/catch 异常。函数出错时返回 Result：
//   Result<T, E> = Ok(T)（成功，带结果）或 Err(E)（失败，带错误信息）。
// `?` 是语法糖：遇到 Err 就直接 return 这个错误给调用者；只有 Ok 才取出
// 里面的值继续往下走。anyhow::Result<T> 是简化版（错误统一装成字符串）。
//
// 【为什么在这里】读游戏内存随时可能失败（地址变了/进程退了），
// 用 Result + ? 让错误一路往上抛，最上层（analyze_all）统一处理。
//
// 【最小示例】
//   fn divide(a: i32, b: i32) -> Result<i32, String> {
//       if b == 0 { return Err("除零".into()); }
//       Ok(a / b)                        // 成功：包进 Ok
//   }
//   fn caller() -> Result<i32, String> {
//       let q = divide(10, 2)?;          // 成功 → q=5；失败 → 直接 return Err
//       Ok(q + 1)
//   }
//
// 【自己动手要点】
// 1) ? 只能在"返回 Result 的函数"里用；
// 2) 需要"出错就崩溃"的场景用 .expect("说明") / unwrap()（示例/测试里常见）；
// 3) 本项目大量 `xxx?` 就是这个机制：把代码从"层层 if 判断"里解放出来。
// ----------------------------------------------------------------------------
fn main() -> Result<()> {
    let args = Args::parse();

    let level_filter = match args.verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![TermLogger::new(
        level_filter,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )];

    // Create the log file by default.
    if !args.no_log_file {
        loggers.push(WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("cs2-dumper.log")?,
        ));
    }

    CombinedLogger::init(loggers)?;

    let conn_args = args
        .connector_args
        .map(|s| ConnectorArgs::from_str(&s).expect("unable to parse connector arguments"))
        .unwrap_or_default();

    let mut os = match args.connector {
        Some(conn) => {
            let mut inventory = Inventory::scan();

            inventory
                .builder()
                .connector(&conn)
                .args(conn_args)
                .os("win32")
                .build()?
        }
        None => {
            #[cfg(windows)]
            {
                memflow_native::create_os(&OsArgs::default(), LibArc::default())?
            }
            #[cfg(not(windows))]
            {
                panic!("no connector specified")
            }
        }
    };

    let mut process = os.process_by_name(&args.process_name)?;

    let now = Instant::now();

    let result = analysis::analyze_all(&mut process)?;
    let output = Output::new(&args.file_types, args.indent_size, &args.output, &result)?;

    output.dump_all(&mut process)?;

    info!("analysis completed in {:.2?}", now.elapsed());

    Ok(())
}
