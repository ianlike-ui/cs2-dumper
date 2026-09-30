// ============================================================================
// output/offsets.rs —— 把"全局偏移"写成各种语言的文件
//
// 与 output/interfaces.rs 几乎一样，只是数据类型是 OffsetMap
// （模块名 → {偏移名 → 值}）。输出的 offsets.hpp 就是 cs2_cheat 里
// external-cheat-base/generated/offsets.hpp 的来源。
// ============================================================================

use std::fmt::{self, Write};

use heck::{AsPascalCase, AsSnakeCase};

use super::{CodeWriter, Formatter, OffsetMap, slugify, zig_ident};

impl CodeWriter for OffsetMap {
    // C# 输出
    fn write_cs(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.block("namespace CS2Dumper.Offsets", false, |fmt| {
            for (module_name, offsets) in self {
                writeln!(fmt, "// Module: {}", module_name)?;

                fmt.block(
                    &format!("public static class {}", AsPascalCase(slugify(module_name))),
                    false,
                    |fmt| {
                        for (name, value) in offsets {
                            writeln!(fmt, "public const nint {} = {:#X};", name, value)?;
                        }

                        Ok(())
                    },
                )?;
            }

            Ok(())
        })
    }

    // C++ 输出
    fn write_hpp(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        writeln!(fmt, "#pragma once\n")?;
        writeln!(fmt, "#include <cstddef>")?;
        writeln!(fmt, "#include <cstdint>\n")?;

        fmt.block("namespace cs2_dumper", false, |fmt| {
            fmt.block("namespace offsets", false, |fmt| {
                for (module_name, offsets) in self {
                    writeln!(fmt, "// Module: {}", module_name)?;

                    fmt.block(
                        &format!("namespace {}", AsSnakeCase(slugify(module_name))),
                        false,
                        |fmt| {
                            for (name, value) in offsets {
                                writeln!(fmt, "constexpr std::ptrdiff_t {} = {:#X};", name, value)?;
                            }

                            Ok(())
                        },
                    )?;
                }

                Ok(())
            })
        })
    }

    // JSON 输出
    fn write_json(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.write_str(&serde_json::to_string_pretty(self).unwrap())
    }

    // Rust 输出
    fn write_rs(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        writeln!(fmt, "#![allow(non_upper_case_globals, unused)]\n")?;

        fmt.block("pub mod cs2_dumper", false, |fmt| {
            fmt.block("pub mod offsets", false, |fmt| {
                for (module_name, offsets) in self {
                    writeln!(fmt, "// Module: {}", module_name)?;

                    fmt.block(
                        &format!("pub mod {}", AsSnakeCase(slugify(module_name))),
                        false,
                        |fmt| {
                            for (name, value) in offsets {
                                writeln!(fmt, "pub const {}: usize = {:#X};", name, value)?;
                            }

                            Ok(())
                        },
                    )?;
                }

                Ok(())
            })
        })
    }

    // Zig 输出
    fn write_zig(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.block("pub const cs2_dumper = struct", true, |fmt| {
            fmt.block("pub const offsets = struct", true, |fmt| {
                for (module_name, offsets) in self {
                    writeln!(fmt, "// Module: {}", module_name)?;

                    let module_name = zig_ident(&AsSnakeCase(slugify(module_name)).to_string());

                    fmt.block(
                        &format!("pub const {} = struct", module_name),
                        true,
                        |fmt| {
                            for (name, value) in offsets {
                                writeln!(
                                    fmt,
                                    "pub const {}: usize = {:#X};",
                                    zig_ident(name),
                                    value
                                )?;
                            }

                            Ok(())
                        },
                    )?;
                }

                Ok(())
            })
        })
    }
}
