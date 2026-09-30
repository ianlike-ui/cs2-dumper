// ============================================================================
// client/input.rs —— "按键"节点的内存布局
//
// 游戏里每个可绑定按键（attack、jump、duck…）对应一个 KeyButton 节点：
//   名字 + 状态 + 链表下一个节点。
// analysis/buttons.rs 顺着链表（next）枚举出所有按键，
// 并记录 state 字段的偏移（外部工具据此读写按键状态）。
// ============================================================================

use memflow::prelude::v1::*;

#[derive(Pod)]
#[repr(C)]
pub struct KeyButton {
    pad_0: [u8; 0x8],                 // 0x0000 对齐填充
    pub name: Pointer64<ReprCString>, // 0x0008 按键名（如 "attack"）
    pad_1: [u8; 0x20],                // 0x0010 对齐填充
    pub state: u32,                   // 0x0030 按键状态
    pad_2: [u8; 0x54],                // 0x0034 对齐填充
    pub next: Pointer64<KeyButton>,   // 0x0088 链表下一个节点
}
