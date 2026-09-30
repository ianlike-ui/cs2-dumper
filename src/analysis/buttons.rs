// ============================================================================
// analysis/buttons.rs —— "按键分析"：枚举游戏按键及其状态地址
//
// 【这个模块是干什么的】
// 在 client.dll 里用特征码找到"按键注册链表"的头，顺着链表读出每个按键
// （attack 开火、jump 跳、duck 蹲…）的名字和"状态变量"的内存偏移。
// 输出就是 buttons.hpp（自动扳机/自动跳跃等功能需要读这些按键状态）。
//
// 【Rust 语法速览】
// - BTreeMap<String, umem>：按键名 → 内存偏移 的有序字典。
// - memflow 的 Pointer64<T>：指向 T 的 64 位指针的封装（从游戏进程内存读取）。
// ============================================================================

use std::collections::BTreeMap;

use anyhow::{Result, bail};

use log::debug;

use memflow::prelude::v1::*;

use pelite::pattern;
use pelite::pe64::{Pe, PeView};

use crate::source2::KeyButton;

pub type ButtonMap = BTreeMap<String, umem>;

pub fn buttons<P: Process + MemoryView>(process: &mut P) -> Result<ButtonMap> {
    let module = process.module_by_name("client.dll")?;

    let buf = process
        .read_raw(module.base, module.size as _)
        .data_part()?;

    let view = PeView::from_bytes(&buf)?;

    let mut save = [0; 2];

    if !view
        .scanner()
        .finds_code(pattern!("488b15${'} 4885d2 74? 488b02 4885c0"), &mut save)
    {
        bail!("outdated button list pattern");
    }

    let list_head = process.read_addr64(module.base + save[1]).data_part()?;

    read_buttons(process, &module, list_head)
}

fn read_buttons(
    mem: &mut impl MemoryView,
    module: &ModuleInfo,
    list_head: Address,
) -> Result<ButtonMap> {
    let mut result = ButtonMap::new();

    let mut button_ptr = Pointer64::<KeyButton>::from(list_head);

    while !button_ptr.is_null() {
        let button = mem.read_ptr(button_ptr).data_part()?;
        let name = mem.read_utf8_lossy(button.name.address(), 32).data_part()?;

        let state_addr = button_ptr.address() + offset_of!(KeyButton.state);

        if let Some(state_rva) = state_addr.to_umem().checked_sub(module.base.to_umem()) {
            debug!(
                "found \"{}\" at {:#X} ({} + {:#X})",
                name,
                state_addr.to_umem(),
                module.name,
                state_rva
            );

            result.insert(name, state_rva);
        }

        button_ptr = button.next;
    }

    Ok(result)
}
