// ============================================================================
// output/buttons.rs —— 把"按键偏移"写成各种语言的文件
//
// 【这个文件是干什么的】
// 为 ButtonMap（按键名 → 偏移）实现 CodeWriter 接口的 5 种语言写法：
//   - C#：public const nint attack = 0x2099000;
//   - C++：constexpr std::ptrdiff_t attack = 0x2099000;
//   - JSON：{ "client.dll": { "attack": ... } }
//   - Rust：pub const attack: usize = 0x2099000;
//   - Zig：pub const attack: usize = 0x2099000;
//
// 【小细节】
// - 写 Rust 时，"use" 是关键字，要转成 r#use（原始标识符）；
// - Zig 用 zig_ident 处理关键字/非法标识符。
// ============================================================================

use std::collections::BTreeMap;
use std::fmt::{self, Write};

use super::{ButtonMap, CodeWriter, Formatter, zig_ident};

impl CodeWriter for ButtonMap {
    // C# 输出
    fn write_cs(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.block("namespace CS2Dumper", false, |fmt| {
            writeln!(fmt, "// Module: client.dll")?;

            fmt.block("public static class Buttons", false, |fmt| {
                for (name, value) in self {
                    writeln!(fmt, "public const nint {} = {:#X};", name, value)?;
                }

                Ok(())
            })
        })
    }

    // C++ 输出
    fn write_hpp(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        writeln!(fmt, "#pragma once\n")?;
        writeln!(fmt, "#include <cstddef>")?;
        writeln!(fmt, "#include <cstdint>\n")?;

        fmt.block("namespace cs2_dumper", false, |fmt| {
            writeln!(fmt, "// Module: client.dll")?;

            fmt.block("namespace buttons", false, |fmt| {
                for (name, value) in self {
                    writeln!(fmt, "constexpr std::ptrdiff_t {} = {:#X};", name, value)?;
                }

                Ok(())
            })
        })
    }

    // JSON 输出
    fn write_json(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        let content = {
            let buttons: BTreeMap<_, _> = self.iter().map(|(name, value)| (name, value)).collect();

            BTreeMap::from_iter([("client.dll", buttons)])
        };

        fmt.write_str(&serde_json::to_string_pretty(&content).unwrap())
    }

    // Rust 输出
    fn write_rs(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        writeln!(fmt, "#![allow(non_upper_case_globals, unused)]\n")?;

        fmt.block("pub mod cs2_dumper", false, |fmt| {
            writeln!(fmt, "// Module: client.dll")?;

            fmt.block("pub mod buttons", false, |fmt| {
                for (name, value) in self {
                    let mut name = name.clone();

                    if name == "use" {
                        name = format!("r#{}", name); // "use" 是 Rust 关键字，用 r# 转义
                    }

                    writeln!(fmt, "pub const {}: usize = {:#X};", name, value)?;
                }

                Ok(())
            })
        })
    }

    // Zig 输出
    fn write_zig(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.block("pub const cs2_dumper = struct", true, |fmt| {
            writeln!(fmt, "// Module: client.dll")?;

            fmt.block("pub const buttons = struct", true, |fmt| {
                for (name, value) in self {
                    writeln!(fmt, "pub const {}: usize = {:#X};", zig_ident(name), value)?;
                }

                Ok(())
            })
        })
    }
}
