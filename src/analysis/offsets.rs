// ============================================================================
// analysis/offsets.rs —— "偏移量分析"：用特征码在游戏 dll 里找全局变量
//
// ----------------------------------------------------------------------------
// 🎓 概念卡 ㊳：特征码扫描（在二进制里搜"指纹"）
// ----------------------------------------------------------------------------
// 【是什么】发布版游戏没有符号表，想知道"某个全局变量在哪"，就用它的
// "机器码指纹"去搜。比如特征码 `488905${'} 0f57c0` 表示一段字节模式：
//   "48 89 05" 后跟任意 4 字节（$ 通配），再跟 "0f 57 c0"。
// pelite 会在整个 dll 里找匹配位置，命中位置附近就是目标变量的地址。
// 特征码要"足够独特"（太短会误命中）且"游戏更新就失效"（代码变了）。
// 【为什么在这里】dwLocalPlayerController、dwViewMatrix 这些全局偏移，
// 全是靠特征码找出来的。这是整个项目的第一个分析，也是经典手法。
// 【最小示例（动手体验：在内存里搜模式）】
//   // 把这段数据看作"dll 的字节"：
//   let data = [0x48u8, 0x89, 0x05, 0x00, 0x00, 0x00, 0x00, 0x0f, 0x57];
//   // 在 data 里找 [48 89 05 ?? ?? ?? ?? 0f 57]：
//   for i in 0..data.len()-8 {
//       if data[i]==0x48 && data[i+1]==0x89 && data[i+2]==0x05
//          && data[i+7]==0x0f && data[i+8]==0x57 { /* 命中在 i */ }
//   }
// 【自己动手要点】
// 1) 特征码来自逆向：用 IDA/Ghidra 找到引用目标变量的指令，抄下字节；
// 2) $ 通配"任意字节"，[$ 长度] 是"定长任意字节"（本项目 offsets 里有例子）；
// 3) 找不到时日志打 "outdated pattern"——提醒该更新特征码了（游戏更新了）。
// ----------------------------------------------------------------------------
// 🎓 概念卡 ㊴：macro_rules! 宏（把重复代码模板化）
// ----------------------------------------------------------------------------
// 【是什么】宏是"代码生成代码"：调用时按模板展开。pattern_map! 一次
// 生成 client/engine2/input_system/... 五个模块，每个模块里有
// 一张 PHF 哈希表和一段扫描逻辑。没有宏就得手抄 5 遍几乎一样的代码。
// 【为什么在这里】五个模块的"找偏移"逻辑完全一样，只有模式表不同——
// 完美符合宏的场景。
// 【最小示例】
//   macro_rules! say {
//       ($name:expr) => { println!("你好，{}", $name); };
//   }
//   say!("世界");      // 展开成 println!("你好，{}", "世界");
// 【自己动手要点】
// 1) 宏的 $xxx:expr 表示"这里可以放一个表达式"；
// 2) 先写普通函数，确认逻辑后再提取成宏，别一上来就写宏；
// 3) 调试宏：cargo expand 可以看展开后的代码。
// ----------------------------------------------------------------------------
//
// 【概念速览】特征码扫描、macro_rules!/phf 宏的详细讲解见上方 🎓 概念卡 ㊳㊴
// （完整索引见《搭建路线图与概念索引.md》）。
// ============================================================================

use std::collections::BTreeMap;

use anyhow::Result;

use log::{debug, error};

use memflow::prelude::v1::*;

use pelite::pattern;
use pelite::pattern::{Atom, save_len};
use pelite::pe64::{Pe, PeView, Rva};

use phf::{Map, phf_map};

pub type OffsetMap = BTreeMap<String, BTreeMap<String, Rva>>;

macro_rules! pattern_map {
    ($($module:ident => {
        $($name:expr => $pattern:expr $(=> $callback:expr)?),+ $(,)?
    }),+ $(,)?) => {
        $(
            mod $module {
                use super::*;

                pub(super) const PATTERNS: Map<
                    &'static str,
                    (
                        &'static [Atom],
                        Option<fn(&PeView, &mut BTreeMap<String, Rva>, Rva)>,
                    ),
                > = phf_map! {
                    $($name => ($pattern, $($callback)?)),+
                };

                pub fn offsets(view: PeView<'_>) -> BTreeMap<String, Rva> {
                    let mut map = BTreeMap::new();

                    for (&name, (pat, callback)) in &PATTERNS {
                        let mut save = vec![0; save_len(pat)];

                        if !view.scanner().finds_code(pat, &mut save) {
                            error!("outdated pattern: {}", name);

                            continue;
                        }

                        let rva = save[1];

                        map.insert(name.to_string(), rva);

                        if let Some(callback) = callback {
                            callback(&view, &mut map, rva);
                        }
                    }

                    for (name, value) in &map {
                        debug!(
                            "found \"{}\" at {:#X} ({}.dll + {:#X})",
                            name,
                            *value as u64 + view.optional_header().ImageBase,
                            stringify!($module),
                            value
                        );
                    }

                    map
                }
            }
        )+
    };
}

pattern_map! {
    client => {
        "dwCSGOInput" => pattern!("488905${'} 0f57c0 0f1105") => Some(|view, map, rva| {
            let mut save = [0; 2];

            if view.scanner().finds_code(pattern!("f2420f108428u4"), &mut save) {
                map.insert("dwViewAngles".to_string(), rva + save[1]);
            }
        }),
        "dwEntityList" => pattern!("48890d${'} e9${} cc") => None,
        "dwGameEntitySystem" => pattern!("488b1d${'} 48891d[4] 4c63b3") => None,
        "dwGameEntitySystem_highestEntityIndex" => pattern!("ff81u4 4885d2") => None,
        "dwGameRules" => pattern!("f6c1010f85${} 4c8b05${'} 4d85") => None,
        "dwGlobalVars" => pattern!("488915${'} 488942") => None,
        "dwGlowManager" => pattern!("488b05${'} c3 cccccccccccccccc 8b41") => None,
        "dwLocalPlayerController" => pattern!("488b05${'} 4189be") => None,
        "dwPlantedC4" => pattern!("488b1d${'} 4532f6") => None,
        "dwPrediction" => pattern!("488d05${'} c3 cccccccccccccccc 405356 4154") => Some(|view, map, rva| {
            let mut save = [0; 2];

            if view.scanner().finds_code(pattern!("4c39b6u4 74? 4488be"), &mut save) {
                map.insert("dwLocalPlayerPawn".to_string(), rva + save[1]);
            }
        }),
        "dwSensitivity" => pattern!("488d0d${[8]'} 0f57c90f28f0") => Some(|_view, map, _rva| {
            map.insert("dwSensitivity_sensitivity".to_string(), 0x58);
        }),
        "dwViewMatrix" => pattern!("488d0d${'} 48c1e006") => None,
        "dwViewRender" => pattern!("488905${'} 488bc8 4885c0") => None,
        "dwWeaponC4" => pattern!("488b15${'} 488b5c24? ffc0 8905${} 488bc6 488934ea 80be") => None,
    },
    engine2 => {
        "dwBuildNumber" => pattern!("8905${'} 488d0d${} ff15${} 488b0d") => None,
        "dwNetworkGameClient" => pattern!("48893d${'} ff87") => None,
        "dwNetworkGameClient_clientTickCount" => pattern!("8b81u4 c3 cccccccccccccccccc 8b81${} c3 cccccccccccccccccc 83b9") => None,
        "dwNetworkGameClient_deltaTick" => pattern!("4c8db7u4 4c897c24") => None,
        "dwNetworkGameClient_isBackgroundMap" => pattern!("0fb681u4 c3 cccccccccccccccc 0fb681${} c3 cccccccccccccccc 4883ec") => None,
        "dwNetworkGameClient_localPlayer" => pattern!("428b94d3u4 5b 49ffe3 32c0 5b c3 cccccccccccccccc 4053") => None,
        "dwNetworkGameClient_maxClients" => pattern!("8b81u4 c3????????? 8b81[4] c3????????? 8b81") => None,
        "dwNetworkGameClient_serverTickCount" => pattern!("8b81u4 c3 cccccccccccccccccc 83b9") => None,
        "dwNetworkGameClient_signOnState" => pattern!("448b81u4 488d0d") => None,
        "dwWindowHeight" => pattern!("8b05${'} 8903") => None,
        "dwWindowWidth" => pattern!("8b05${'} 8907") => None,
    },
    input_system => {
        "dwInputSystem" => pattern!("488905${'} 33c0") => None,
    },
    matchmaking => {
        "dwGameTypes" => pattern!("488d0d${'} ff90") => None,
    },
    soundsystem => {
        "dwSoundSystem" => pattern!("488d0d${'} e8${} 488b0d${} [3] 4c8b82") => None,
        "dwSoundSystem_engineViewData" => pattern!("0f1147u1 0f104f10 0f114f7c") => None,
    },
}

pub fn offsets<P: Process + MemoryView>(process: &mut P) -> Result<OffsetMap> {
    let mut map = BTreeMap::new();

    let modules: [(&str, fn(PeView) -> BTreeMap<String, u32>); 5] = [
        ("client.dll", client::offsets),
        ("engine2.dll", engine2::offsets),
        ("inputsystem.dll", input_system::offsets),
        ("matchmaking.dll", matchmaking::offsets),
        ("soundsystem.dll", soundsystem::offsets),
    ];

    for (module_name, offsets) in &modules {
        let module = process.module_by_name(module_name)?;

        let buf = process
            .read_raw(module.base, module.size as _)
            .data_part()?;

        let view = PeView::from_bytes(&buf)?;

        map.insert(module_name.to_string(), offsets(view));
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Once;

    use serde_json::Value;

    use simplelog::*;

    use super::*;

    #[test]
    fn build_number() -> Result<()> {
        let mut process = setup()?;

        let engine_base = process.module_by_name("engine2.dll")?.base;

        let offset = read_offset("engine2.dll", "dwBuildNumber").unwrap();

        let build_number: u32 = process.read(engine_base + offset).data_part()?;

        debug!("build number: {}", build_number);

        Ok(())
    }

    #[test]
    fn global_vars() -> Result<()> {
        let mut process = setup()?;

        let client_base = process.module_by_name("client.dll")?.base;

        let offset = read_offset("client.dll", "dwGlobalVars").unwrap();

        let global_vars: u64 = process.read(client_base + offset).data_part()?;

        let map_name_addr = process
            .read_addr64((global_vars + 0x180).into())
            .data_part()?;

        let map_name = process.read_utf8(map_name_addr, 128).data_part()?;

        debug!("[global vars] map name: \"{}\"", map_name);

        Ok(())
    }

    #[test]
    fn local_controller() -> Result<()> {
        let mut process = setup()?;

        let client_base = process.module_by_name("client.dll")?.base;

        let local_controller_offset = read_offset("client.dll", "dwLocalPlayerController").unwrap();

        let player_name_offset =
            read_class_field("client.dll", "CBasePlayerController", "m_iszPlayerName").unwrap();

        let local_controller: u64 = process
            .read(client_base + local_controller_offset)
            .data_part()?;

        let player_name = process
            .read_utf8((local_controller + player_name_offset).into(), 128)
            .data_part()?;

        debug!("[local controller] name: \"{}\"", player_name);

        Ok(())
    }

    #[test]
    fn local_pawn() -> Result<()> {
        #[derive(Pod)]
        #[repr(C)]
        struct Vector3D {
            x: f32,
            y: f32,
            z: f32,
        }

        let mut process = setup()?;

        let client_base = process.module_by_name("client.dll")?.base;

        let local_player_pawn_offset = read_offset("client.dll", "dwLocalPlayerPawn").unwrap();

        let game_scene_node_offset =
            read_class_field("client.dll", "C_BaseEntity", "m_pGameSceneNode").unwrap();

        let origin_offset =
            read_class_field("client.dll", "CGameSceneNode", "m_vecAbsOrigin").unwrap();

        let local_player_pawn: u64 = process
            .read(client_base + local_player_pawn_offset)
            .data_part()?;

        let game_scene_node: u64 = process
            .read((local_player_pawn + game_scene_node_offset).into())
            .data_part()?;

        let origin: Vector3D = process
            .read((game_scene_node + origin_offset).into())
            .data_part()?;

        debug!(
            "[local pawn] origin: {:.2}, y: {:.2}, z: {:.2}",
            origin.x, origin.y, origin.z
        );

        Ok(())
    }

    #[test]
    fn window_size() -> Result<()> {
        let mut process = setup()?;

        let engine_base = process.module_by_name("engine2.dll")?.base;

        let window_width_offset = read_offset("engine2.dll", "dwWindowWidth").unwrap();
        let window_height_offset = read_offset("engine2.dll", "dwWindowHeight").unwrap();

        let window_width: u32 = process
            .read(engine_base + window_width_offset)
            .data_part()?;

        let window_height: u32 = process
            .read(engine_base + window_height_offset)
            .data_part()?;

        debug!("window size: {}x{}", window_width, window_height);

        Ok(())
    }

    fn setup() -> Result<IntoProcessInstanceArcBox<'static>> {
        static LOGGER: Once = Once::new();

        LOGGER.call_once(|| {
            SimpleLogger::init(LevelFilter::Trace, Config::default()).ok();
        });

        let os = memflow_native::create_os(&OsArgs::default(), LibArc::default())?;

        let process = os.into_process_by_name("cs2.exe")?;

        Ok(process)
    }

    fn read_class_field(module_name: &str, class_name: &str, field_name: &str) -> Option<u64> {
        let content =
            fs::read_to_string(format!("output/{}.json", module_name.replace(".", "_"))).ok()?;

        let value: Value = serde_json::from_str(&content).ok()?;

        value
            .get(module_name)?
            .get("classes")?
            .get(class_name)?
            .get("fields")?
            .get(field_name)?
            .as_u64()
    }

    fn read_offset(module_name: &str, offset_name: &str) -> Option<u64> {
        let content = fs::read_to_string("output/offsets.json").ok()?;
        let value: Value = serde_json::from_str(&content).ok()?;

        let offset = value.get(module_name)?.get(offset_name)?;

        offset.as_u64()
    }
}
