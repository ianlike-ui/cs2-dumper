// ============================================================================
// output/formatter.rs —— 代码生成用的"格式化器"
//
// ----------------------------------------------------------------------------
// 🎓 概念卡 ㊵：trait（接口）与生命周期 'a（借用多久）
// ----------------------------------------------------------------------------
// 【trait 是什么】类似 C++ 的"接口/抽象基类"：定义一组必须实现的方法。
// `impl Write for Formatter` 意思是"让 Formatter 这个类型实现标准库的
// Write 接口"——实现后就能用 writeln!(fmt, ...) 写它。
// derive（如 #[derive(Pod)]）是"自动实现 trait"的快捷方式：编译器帮你
// 照着字段逐个实现，不用手写。
// 【生命周期 'a 是什么】Rust 的所有权规则："一个值只有一个主人，借用必须
// 有期限"。`Formatter<'a>` 里的 'a 表示"我借用的那个 String，至少活得和
// 我一样久"。这是编译期检查的"借用期限声明"，防止悬空引用。
// 【最小示例】
//   struct Wrapper<'a> { inner: &'a mut String }   // 借一个 String，期限 'a
//   impl<'a> Wrapper<'a> {
//       fn push(&mut self, s: &str) { self.inner.push_str(s); }
//   }
//   fn main() {
//       let mut buf = String::new();
//       let mut w = Wrapper { inner: &mut buf };   // buf 必须比 w 活得久
//       w.push("hi");
//   }
// 【自己动手要点】
// 1) 初学阶段把 'a 理解成"借用关系由编译器把关"，写代码时让借用的东西
//    活得足够长（比如在同一个作用域），编译器报错就照提示修；
// 2) trait 方法用 impl Trait for Type 实现，一个类型可以实现多个 trait；
// 3) derive 能自动生成的（Debug/Clone/Pod…）就别手写。
// ----------------------------------------------------------------------------
//
// 【这个类是干什么的】
// 生成各种语言的代码时，缩进是最麻烦的（每个语言风格不同）。
// Formatter 负责：
//   - 跟踪当前的缩进层级（indent_level）；
//   - block()：生成 `名字 { ... }` 这样的代码块（支持结尾 `};`）；
//   - 实现 Rust 的 Write trait：往输出字符串写内容时自动加缩进。
//
// 【Rust 语法速览】
// - impl Write for Formatter：让 Formatter 可以作为标准库的"写目标"
//   （writeln!(fmt, ...) 就能往里面写，自动缩进）。
// - 生命周期 'a：表示 Formatter 借用了一个外部的 String（不拥有它）。
// ============================================================================

use std::fmt::{self, Write};

// 代码格式化器：包装一个 String，自动处理缩进
pub struct Formatter<'a> {
    out: &'a mut String,   // 输出目标（借用外部字符串）
    indent_size: usize,    // 每级缩进用几个空格
    indent_level: usize,   // 当前缩进层级
}

impl<'a> Formatter<'a> {
    // 创建格式化器
    pub fn new(out: &'a mut String, indent_size: usize) -> Self {
        Self {
            out,
            indent_size,
            indent_level: 0,
        }
    }

    // 生成一个代码块：`heading { ... }`，semicolon 控制结尾是 "};" 还是 "}"
    // 参数 f 是一个闭包，在里面写块内容（期间缩进自动 +1）
    // TODO: Refactor this.
    pub fn block<F>(&mut self, heading: &str, semicolon: bool, f: F) -> fmt::Result
    where
        F: FnOnce(&mut Self) -> fmt::Result,
    {
        writeln!(self, "{} {{", heading)?;

        self.indent(f)?;

        writeln!(self, "{}", if semicolon { "};" } else { "}" })?;

        Ok(())
    }

    // 在闭包执行期间缩进 +1，结束恢复
    pub fn indent<F>(&mut self, f: F) -> fmt::Result
    where
        F: FnOnce(&mut Self) -> fmt::Result,
    {
        self.indent_level += 1;

        f(self)?;

        self.indent_level -= 1;

        Ok(())
    }

    // 在行首推入当前缩进（indent_level × indent_size 个空格）
    #[inline]
    fn push_indentation(&mut self) {
        if self.indent_level > 0 {
            let indentation = " ".repeat(self.indent_level * self.indent_size);

            self.out.push_str(&indentation);
        }
    }
}

// 实现标准库的 Write trait：write_str 时按"行"处理，新行自动加缩进
impl<'a> Write for Formatter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut lines = s.lines().peekable();

        while let Some(line) = lines.next() {
            // 如果当前在行首且这行非空，先加缩进
            if self.out.ends_with('\n') && !line.is_empty() {
                self.push_indentation();
            }

            self.out.push_str(line);

            // 行与行之间补换行（保持原文的换行习惯）
            if lines.peek().is_some() || s.ends_with('\n') {
                self.out.push('\n');
            }
        }

        Ok(())
    }
}
