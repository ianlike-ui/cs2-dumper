// ============================================================================
// analysis/interfaces.rs —— "接口分析"：枚举模块导出的 CreateInterface 接口
//
// 【什么是接口（Interface）？】
// 游戏引擎（Source 2）里，很多子系统对外暴露一个"接口"——一组可调用函数。
// 比如引擎的"网络客户端"接口。程序想调用它们，得先知道接口实例在内存的哪里。
// 每个 dll 都导出一个 CreateInterface 函数，顺着它返回的接口注册链表
// （InterfaceReg 单链表）就能枚举出所有接口的名字和地址。
//
// 【流程】
//   1. 遍历进程的所有模块（dll）；
//   2. 每个模块找到 CreateInterface 导出符号；
//   3. 解析 RIP 相对寻址，得到接口链表头；
//   4. 顺着链表读出每个接口的名字和实例地址（相对模块基地址的偏移）。
// ============================================================================

use std::collections::BTreeMap;

use anyhow::Result;

use log::debug;

use memflow::prelude::v1::*;

use pelite::pe64::exports::Export;
use pelite::pe64::{Pe, PeView};

use crate::memory::address;
use crate::source2::InterfaceReg;

pub type InterfaceMap = BTreeMap<String, BTreeMap<String, umem>>;

pub fn interfaces<P: Process + MemoryView>(process: &mut P) -> Result<InterfaceMap> {
    process
        .module_list()?
        .iter()
        .filter_map(|module| {
            let buf = process
                .read_raw(module.base, module.size as _)
                .data_part()
                .ok()?;

            let view = PeView::from_bytes(&buf).ok()?;

            let ci_export = view
                .exports()
                .ok()?
                .by()
                .ok()?
                .name("CreateInterface")
                .ok()?;

            match ci_export {
                Export::Symbol(symbol) => {
                    let list_ptr = address::resolve_rip(process, module.base + symbol).ok()?;
                    let list_head = process.read_addr64(list_ptr).data_part().ok()?;

                    return read_interfaces(process, module, list_head)
                        .ok()
                        .filter(|ifaces| !ifaces.is_empty())
                        .map(|ifaces| Ok((module.name.to_string(), ifaces)));
                }
                _ => None,
            }
        })
        .collect()
}

fn read_interfaces(
    mem: &mut impl MemoryView,
    module: &ModuleInfo,
    list_head: Address,
) -> Result<BTreeMap<String, umem>> {
    let mut result = BTreeMap::new();

    let mut reg_ptr = Pointer64::<InterfaceReg>::from(list_head);

    while !reg_ptr.is_null() {
        let reg = mem.read_ptr(reg_ptr).data_part()?;
        let name = mem.read_utf8_lossy(reg.name.address(), 128).data_part()?;

        let instance_addr = address::resolve_rip(mem, reg.create_fn.address())?;

        if let Some(instance_rva) = instance_addr.to_umem().checked_sub(module.base.to_umem()) {
            debug!(
                "found \"{}\" at {:#X} ({} + {:#X})",
                name,
                instance_addr.to_umem(),
                module.name,
                instance_rva
            );

            result.insert(name, instance_rva);
        }

        reg_ptr = reg.next;
    }

    Ok(result)
}
