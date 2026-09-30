// ============================================================================
// output/schemas.rs —— 把"Schema 类表"写成各种语言的文件（最大的一份输出）
//
// 【这个文件是干什么的】
// 把 SchemaMap（模块名 → (类列表, 枚举列表)）写成 5 种语言。
// 生成的 client_dll.hpp 就是 cs2_cheat 里用的那份：每个类是一个
// namespace，里面每个字段一行 `constexpr std::ptrdiff_t 字段名 = 偏移;`。
//
// 【生成时处理的各种边角情况】
// - 枚举的底层类型由 alignment（对齐）决定：1→u8、2→u16、4→u32、8→u64；
// - 枚举成员值可能超出该类型能表达的范围：C# 用 unchecked 强制转换，
//   C++ 直接截断到该类型最大值，Zig 做无符号回绕（format_zig_enum_member_value）；
// - Rust/Zig 里重复的枚举值会被跳过（语言不允许重复判别值）；
// - 类名/字段名统一做 slugify（去掉非法字符），Zig 再包一层 zig_ident；
// - 元数据（网络变量等）写进注释里（write_metadata），方便人读。
// ============================================================================

use std::collections::{BTreeMap, HashSet};
use std::fmt::{self, Write};

use heck::{AsPascalCase, AsSnakeCase};

use serde_json::json;

use super::{CodeWriter, Formatter, SchemaMap, slugify, zig_ident};

use crate::analysis::ClassMetadata;

impl CodeWriter for SchemaMap {
    fn write_cs(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.block("namespace CS2Dumper.Schemas", false, |fmt| {
            for (module_name, (classes, enums)) in self {
                writeln!(fmt, "// Module: {}", module_name)?;
                writeln!(fmt, "// Class count: {}", classes.len())?;
                writeln!(fmt, "// Enum count: {}", enums.len())?;

                fmt.block(
                    &format!("public static class {}", AsPascalCase(slugify(module_name))),
                    false,
                    |fmt| {
                        for enum_ in enums {
                            let type_name = match enum_.alignment {
                                1 => "byte",
                                2 => "ushort",
                                4 => "uint",
                                8 => "ulong",
                                _ => continue,
                            };

                            writeln!(fmt, "// Alignment: {}", enum_.alignment)?;
                            writeln!(fmt, "// Member count: {}", enum_.size)?;

                            fmt.block(
                                &format!("public enum {} : {}", slugify(&enum_.name), type_name),
                                false,
                                |fmt| {
                                    let members = enum_
                                        .members
                                        .iter()
                                        .map(|member| {
                                            let formatted_value =
                                                if (0..=i32::MAX as i64).contains(&member.value) {
                                                    format!("{:#X}", member.value)
                                                } else {
                                                    format!(
                                                        "unchecked(({}){})",
                                                        type_name, member.value
                                                    )
                                                };

                                            format!("{} = {}", member.name, formatted_value)
                                        })
                                        .collect::<Vec<_>>()
                                        .join(",\n");

                                    writeln!(fmt, "{}", members)
                                },
                            )?;
                        }

                        for class in classes {
                            let parent_name = class
                                .parent_name
                                .as_deref()
                                .map(slugify)
                                .unwrap_or("None".to_string());

                            writeln!(fmt, "// Parent: {}", parent_name)?;
                            writeln!(fmt, "// Field count: {}", class.fields.len())?;

                            write_metadata(fmt, &class.metadata)?;

                            fmt.block(
                                &format!("public static class {}", slugify(&class.name)),
                                false,
                                |fmt| {
                                    for field in &class.fields {
                                        writeln!(
                                            fmt,
                                            "public const nint {} = {:#X}; // {}",
                                            field.name, field.offset, field.type_name
                                        )?;
                                    }

                                    Ok(())
                                },
                            )?;
                        }

                        Ok(())
                    },
                )?;
            }

            Ok(())
        })
    }

    fn write_hpp(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        writeln!(fmt, "#pragma once\n")?;
        writeln!(fmt, "#include <cstddef>")?;
        writeln!(fmt, "#include <cstdint>\n")?;

        fmt.block("namespace cs2_dumper", false, |fmt| {
            fmt.block("namespace schemas", false, |fmt| {
                for (module_name, (classes, enums)) in self {
                    writeln!(fmt, "// Module: {}", module_name)?;
                    writeln!(fmt, "// Class count: {}", classes.len())?;
                    writeln!(fmt, "// Enum count: {}", enums.len())?;

                    fmt.block(
                        &format!("namespace {}", AsSnakeCase(slugify(module_name))),
                        false,
                        |fmt| {
                            for enum_ in enums {
                                let type_name = match enum_.alignment {
                                    1 => "uint8_t",
                                    2 => "uint16_t",
                                    4 => "uint32_t",
                                    8 => "uint64_t",
                                    _ => continue,
                                };

                                writeln!(fmt, "// Alignment: {}", enum_.alignment)?;
                                writeln!(fmt, "// Member count: {}", enum_.size)?;

                                fmt.block(
                                    &format!("enum class {} : {}", slugify(&enum_.name), type_name),
                                    true,
                                    |fmt| {
                                        let members = enum_
                                            .members
                                            .iter()
                                            .map(|member| {
                                                let formatted_value = if (0..=i32::MAX as i64)
                                                    .contains(&member.value)
                                                {
                                                    format!("{:#X}", member.value)
                                                } else {
                                                    let max_value = match type_name {
                                                        "uint8_t" => 0xFFu64,
                                                        "uint16_t" => 0xFFFFu64,
                                                        "uint32_t" => 0xFFFFFFFFu64,
                                                        "uint64_t" => 0xFFFFFFFFFFFFFFFFu64,
                                                        _ => 0,
                                                    };

                                                    format!("{:#X}", max_value)
                                                };

                                                format!("{} = {}", member.name, formatted_value)
                                            })
                                            .collect::<Vec<_>>()
                                            .join(",\n");

                                        writeln!(fmt, "{}", members)
                                    },
                                )?;
                            }

                            for class in classes {
                                let parent_name = class
                                    .parent_name
                                    .as_deref()
                                    .map(slugify)
                                    .unwrap_or("None".to_string());

                                writeln!(fmt, "// Parent: {}", parent_name)?;
                                writeln!(fmt, "// Field count: {}", class.fields.len())?;

                                write_metadata(fmt, &class.metadata)?;

                                fmt.block(
                                    &format!("namespace {}", slugify(&class.name)),
                                    false,
                                    |fmt| {
                                        for field in &class.fields {
                                            writeln!(
                                                fmt,
                                                "constexpr std::ptrdiff_t {} = {:#X}; // {}",
                                                field.name, field.offset, field.type_name
                                            )?;
                                        }

                                        Ok(())
                                    },
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

    fn write_json(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        let content: BTreeMap<_, _> = self
            .iter()
            .map(|(module_name, (classes, enums))| {
                let classes: BTreeMap<_, _> = classes
                    .iter()
                    .map(|class| {
                        let fields: BTreeMap<_, _> = class
                            .fields
                            .iter()
                            .map(|field| (&field.name, field.offset))
                            .collect();

                        let metadata: Vec<_> = class
                            .metadata
                            .iter()
                            .map(|metadata| match metadata {
                                ClassMetadata::NetworkChangeCallback { name } => json!({
                                    "type": "NetworkChangeCallback",
                                    "name": name,
                                }),
                                ClassMetadata::NetworkVarNames { name, type_name } => json!({
                                    "type": "NetworkVarNames",
                                    "name": name,
                                    "type_name": type_name,
                                }),
                                ClassMetadata::Unknown { name } => json!({
                                    "type": "Unknown",
                                    "name": name,
                                }),
                            })
                            .collect();

                        (
                            slugify(&class.name),
                            json!({
                                "parent": class.parent_name,
                                "fields": fields,
                                "metadata": metadata
                            }),
                        )
                    })
                    .collect();

                let enums: BTreeMap<_, _> = enums
                    .iter()
                    .map(|enum_| {
                        let members: BTreeMap<_, _> = enum_
                            .members
                            .iter()
                            .map(|member| (&member.name, member.value))
                            .collect();

                        let type_name = match enum_.alignment {
                            1 => "uint8",
                            2 => "uint16",
                            4 => "uint32",
                            8 => "uint64",
                            _ => "unknown",
                        };

                        (
                            slugify(&enum_.name),
                            json!({
                                "alignment": enum_.alignment,
                                "type": type_name,
                                "members": members,
                            }),
                        )
                    })
                    .collect();

                (
                    module_name,
                    json!({
                        "classes": classes,
                        "enums": enums,
                    }),
                )
            })
            .collect();

        fmt.write_str(&serde_json::to_string_pretty(&content).unwrap())
    }

    fn write_rs(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            fmt,
            "#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, unused)]\n"
        )?;

        fmt.block("pub mod cs2_dumper", false, |fmt| {
            fmt.block("pub mod schemas", false, |fmt| {
                for (module_name, (classes, enums)) in self {
                    writeln!(fmt, "// Module: {}", module_name)?;
                    writeln!(fmt, "// Class count: {}", classes.len())?;
                    writeln!(fmt, "// Enum count: {}", enums.len())?;

                    fmt.block(
                        &format!("pub mod {}", AsSnakeCase(slugify(module_name))),
                        false,
                        |fmt| {
                            for enum_ in enums {
                                let type_name = match enum_.alignment {
                                    1 => "u8",
                                    2 => "u16",
                                    4 => "u32",
                                    8 => "u64",
                                    _ => continue,
                                };

                                writeln!(fmt, "// Alignment: {}", enum_.alignment)?;
                                writeln!(fmt, "// Member count: {}", enum_.size)?;

                                fmt.block(
                                    &format!(
                                        "#[repr({})]\npub enum {}",
                                        type_name,
                                        slugify(&enum_.name),
                                    ),
                                    false,
                                    |fmt| {
                                        let mut used_values = HashSet::new();

                                        let members = enum_
                                            .members
                                            .iter()
                                            .filter_map(|member| {
                                                // Skip duplicate values.
                                                if !used_values.insert(member.value) {
                                                    return None;
                                                }

                                                let formatted_value = if member.value == -1 {
                                                    format!("{}::MAX", type_name)
                                                } else {
                                                    format!("{:#X}", member.value)
                                                };

                                                Some(format!(
                                                    "{} = {}",
                                                    member.name, formatted_value
                                                ))
                                            })
                                            .collect::<Vec<_>>()
                                            .join(",\n");

                                        writeln!(fmt, "{}", members)
                                    },
                                )?;
                            }

                            for class in classes {
                                let parent_name = class
                                    .parent_name
                                    .as_deref()
                                    .map(slugify)
                                    .unwrap_or("None".to_string());

                                writeln!(fmt, "// Parent: {}", parent_name)?;
                                writeln!(fmt, "// Field count: {}", class.fields.len())?;

                                write_metadata(fmt, &class.metadata)?;

                                fmt.block(
                                    &format!("pub mod {}", slugify(&class.name)),
                                    false,
                                    |fmt| {
                                        for field in &class.fields {
                                            writeln!(
                                                fmt,
                                                "pub const {}: usize = {:#X}; // {}",
                                                field.name, field.offset, field.type_name
                                            )?;
                                        }

                                        Ok(())
                                    },
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

    fn write_zig(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.block("pub const cs2_dumper = struct", true, |fmt| {
            fmt.block("pub const schemas = struct", true, |fmt| {
                for (module_name, (classes, enums)) in self {
                    writeln!(fmt, "// Module: {}", module_name)?;
                    writeln!(fmt, "// Class count: {}", classes.len())?;
                    writeln!(fmt, "// Enum count: {}", enums.len())?;

                    let module_name = zig_ident(&AsSnakeCase(slugify(module_name)).to_string());

                    fmt.block(
                        &format!("pub const {} = struct", module_name),
                        true,
                        |fmt| {
                            for enum_ in enums {
                                let type_name = match enum_.alignment {
                                    1 => "u8",
                                    2 => "u16",
                                    4 => "u32",
                                    8 => "u64",
                                    _ => continue,
                                };

                                writeln!(fmt, "// Alignment: {}", enum_.alignment)?;
                                writeln!(fmt, "// Member count: {}", enum_.size)?;

                                let enum_name = zig_ident(&slugify(&enum_.name));

                                fmt.block(
                                    &format!("pub const {} = enum({})", enum_name, type_name),
                                    true,
                                    |fmt| {
                                        let mut used_values = HashSet::new();

                                        let members = enum_
                                            .members
                                            .iter()
                                            .filter_map(|member| {
                                                // Skip duplicate values.
                                                if !used_values.insert(member.value) {
                                                    return None;
                                                }

                                                let formatted_value = format_zig_enum_member_value(
                                                    member.value,
                                                    type_name,
                                                );

                                                Some(format!(
                                                    "{} = {}",
                                                    zig_ident(&member.name),
                                                    formatted_value
                                                ))
                                            })
                                            .collect::<Vec<_>>()
                                            .join(",\n");

                                        writeln!(fmt, "{}", members)
                                    },
                                )?;
                            }

                            for class in classes {
                                let parent_name = class
                                    .parent_name
                                    .as_deref()
                                    .map(slugify)
                                    .unwrap_or("None".to_string());

                                writeln!(fmt, "// Parent: {}", parent_name)?;
                                writeln!(fmt, "// Field count: {}", class.fields.len())?;

                                write_metadata(fmt, &class.metadata)?;

                                let class_name = zig_ident(&slugify(&class.name));

                                fmt.block(
                                    &format!("pub const {} = struct", class_name),
                                    true,
                                    |fmt| {
                                        for field in &class.fields {
                                            writeln!(
                                                fmt,
                                                "pub const {}: usize = {:#X}; // {}",
                                                zig_ident(&field.name),
                                                field.offset,
                                                field.type_name
                                            )?;
                                        }

                                        Ok(())
                                    },
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

// 把类的元数据（网络变量等）写成注释（每种语言通用）
fn write_metadata(fmt: &mut Formatter<'_>, metadata: &[ClassMetadata]) -> fmt::Result {
    if metadata.is_empty() {
        return Ok(());
    }

    writeln!(fmt, "//")?;
    writeln!(fmt, "// Metadata:")?;

    for metadata in metadata {
        match metadata {
            ClassMetadata::NetworkChangeCallback { name } => {
                writeln!(fmt, "// NetworkChangeCallback: {}", name)?;
            }
            ClassMetadata::NetworkVarNames { name, type_name } => {
                writeln!(fmt, "// NetworkVarNames: {} ({})", name, type_name)?;
            }
            ClassMetadata::Unknown { name } => {
                writeln!(fmt, "// {}", name)?;
            }
        }
    }

    Ok(())
}

// Zig 枚举成员值格式化：负数按目标无符号类型"回绕"成 16 进制
fn format_zig_enum_member_value(value: i64, type_name: &str) -> String {
    if value >= 0 {
        return format!("{:#X}", value);
    }

    let wrapped_value = match type_name {
        "u8" => value as u8 as u64,
        "u16" => value as u16 as u64,
        "u32" => value as u32 as u64,
        "u64" => value as u64,
        _ => 0,
    };

    format!("{:#X}", wrapped_value)
}
