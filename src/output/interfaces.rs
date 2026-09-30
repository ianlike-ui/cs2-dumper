// ============================================================================
// output/interfaces.rs —— 把"接口偏移"写成各种语言的文件
//
// 与 output/buttons.rs 类似，只是数据类型是 InterfaceMap
// （模块名 → {接口名 → 偏移}）。额外处理：
//   - 模块名用 slugify + heck 转成各语言合法的标识符
//     （比如 client.dll → client_dll / ClientDll）；
//   - C# 里偏移超过 i32 上限时，用 unchecked((nint)...) 强制转换
//     （否则编译报错，因为常量太大了）。
// ============================================================================

use std::collections::BTreeMap;
use std::fmt::{self, Write};

use heck::{AsPascalCase, AsSnakeCase};

use super::{CodeWriter, Formatter, InterfaceMap, slugify, zig_ident};

impl CodeWriter for InterfaceMap {
    // C# 输出
    fn write_cs(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.block("namespace CS2Dumper.Interfaces", false, |fmt| {
            for (module_name, ifaces) in self {
                writeln!(fmt, "// Module: {}", module_name)?;

                fmt.block(
                    &format!("public static class {}", AsPascalCase(slugify(module_name))),
                    false,
                    |fmt| {
                        for (name, value) in ifaces {
                            // 偏移超过 int 上限时，C# 需要 unchecked 转换
                            if *value > i32::MAX as u64 {
                                writeln!(
                                    fmt,
                                    "public static readonly nint {} = unchecked((nint){:#X});",
                                    name, value
                                )?;
                            } else {
                                writeln!(fmt, "public const nint {} = {:#X};", name, value)?;
                            };
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
            fmt.block("namespace interfaces", false, |fmt| {
                for (module_name, ifaces) in self {
                    writeln!(fmt, "// Module: {}", module_name)?;

                    fmt.block(
                        &format!("namespace {}", AsSnakeCase(slugify(module_name))),
                        false,
                        |fmt| {
                            for (name, value) in ifaces {
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
        let content: BTreeMap<_, _> = self
            .iter()
            .map(|(module_name, ifaces)| {
                let ifaces: BTreeMap<_, _> =
                    ifaces.iter().map(|(name, value)| (name, value)).collect();

                (module_name, ifaces)
            })
            .collect();

        fmt.write_str(&serde_json::to_string_pretty(&content).unwrap())
    }

    // Rust 输出
    fn write_rs(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        writeln!(fmt, "#![allow(non_upper_case_globals, unused)]\n")?;

        fmt.block("pub mod cs2_dumper", false, |fmt| {
            fmt.block("pub mod interfaces", false, |fmt| {
                for (module_name, ifaces) in self {
                    writeln!(fmt, "// Module: {}", module_name)?;

                    fmt.block(
                        &format!("pub mod {}", AsSnakeCase(slugify(module_name))),
                        false,
                        |fmt| {
                            for (name, value) in ifaces {
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
            fmt.block("pub const interfaces = struct", true, |fmt| {
                for (module_name, ifaces) in self {
                    writeln!(fmt, "// Module: {}", module_name)?;

                    let module_name = zig_ident(&AsSnakeCase(slugify(module_name)).to_string());

                    fmt.block(
                        &format!("pub const {} = struct", module_name),
                        true,
                        |fmt| {
                            for (name, value) in ifaces {
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
