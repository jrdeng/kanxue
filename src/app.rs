use crate::win32;
use config::Config;
use eframe::egui;
use log::{debug, error, info};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Default)]
struct State {
    handle: i64,
    addr_name: i64,
    addr_battle_info: i64,
    addr_battle_flag: i64,
    characters: HashMap<i32, Character>,
    ctx: Option<egui::Context>,
}

pub struct MyApp {
    hwnd: String,
    state: Arc<Mutex<State>>,
}

#[derive(Debug, Clone)]
pub struct Character {
    pub pos_in_grid: i32, // this is not the index in memory
    pub name: String,
    pub lv: i32,
    pub hp: i32,
    pub hp_max: i32,
    pub mp: i32,
    pub mp_max: i32,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            pos_in_grid: -1,
            name: "".to_owned(),
            lv: 0,
            hp: 0,
            hp_max: 0,
            mp: 0,
            mp_max: 0,
        }
    }
}

impl Character {
    pub fn from(character_info: &str) -> Self {
        // index|name|cuid|?|lv|hp|hp_max|mp|mp_max|cid?|?|?
        let mut character = Character::default();
        let character_info_vec: Vec<&str> = character_info.split('|').collect();
        if character_info_vec.len() < 12 {
            error!("Character info is not enough");
            return character;
        }
        let index = i32::from_str_radix(character_info_vec[0], 16).unwrap();
        // transform index to pos in grid
        character.pos_in_grid = match index {
            14 => 0,
            12 => 1,
            10 => 2,
            11 => 3,
            13 => 4,
            19 => 5,
            17 => 6,
            15 => 7,
            16 => 8,
            18 => 9,
            9 => 10,
            7 => 11,
            5 => 12,
            6 => 13,
            8 => 14,
            4 => 15,
            2 => 16,
            0 => 17,
            1 => 18,
            3 => 19,
            _ => -1,
        };
        character.name = character_info_vec[1].to_owned();
        character.lv = i32::from_str_radix(character_info_vec[4], 16).unwrap();
        character.hp = i32::from_str_radix(character_info_vec[5], 16).unwrap();
        character.hp_max = i32::from_str_radix(character_info_vec[6], 16).unwrap();
        character.mp = i32::from_str_radix(character_info_vec[7], 16).unwrap();
        character.mp_max = i32::from_str_radix(character_info_vec[8], 16).unwrap();
        character
    }
}

fn setup_custom_fonts(ctx: &egui::Context, font_path: String) {
    if let Ok(font) = std::fs::read(&font_path) {
        let mut fonts = egui::FontDefinitions::default();
        fonts
            .font_data
            .insert("my_font".to_owned(), egui::FontData::from_owned(font));

        // Put my font first (highest priority) for proportional text:
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "my_font".to_owned());

        // Put my font as last fallback for monospace:
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("my_font".to_owned());

        // Tell egui to use these fonts:
        ctx.set_fonts(fonts);
    } else {
        error!("failed to open font_path: {font_path}");
    }
}

fn read_shared_handle(state: &Arc<Mutex<State>>) -> i64 {
    state.lock().unwrap().handle
}

fn write_shared_handle(state: &mut Arc<Mutex<State>>, new_value: i64) {
    state.lock().unwrap().handle = new_value;
}

fn read_shared_characters(state: &Arc<Mutex<State>>) -> HashMap<i32, Character> {
    state.lock().unwrap().characters.clone()
}

fn write_shared_characters(state: &mut Arc<Mutex<State>>, new_value: HashMap<i32, Character>) {
    state.lock().unwrap().characters = new_value;
}

fn get_battle_info_thread(state: Arc<Mutex<State>>) {
    loop {
        // update interval = 1s
        std::thread::sleep(std::time::Duration::from_millis(500));

        let tmp_handle = read_shared_handle(&state);

        if tmp_handle == -999 {
            info!("get_battle_info_thread found handle == -999m will exit");
            break;
        } else if tmp_handle > 0 {
            debug!("get_battle_info_thread handle={}", tmp_handle);
            let tmp_addr_battle_info;
            {
                tmp_addr_battle_info = state.lock().unwrap().addr_battle_info;
            }
            let battle_info = win32::read_memory_as_string(tmp_handle, tmp_addr_battle_info, 1000);
            // debug!("get_battle_info_thread battle_info={}", battle_info);
            if battle_info.len() > 0 {
                // index|name|cuid|?|lv|hp|hp_max|mp|mp_max|cid?|?|?|        (12 fields, repeat)
                let fields: Vec<&str> = battle_info.split('|').collect();
                let mut characters_info: Vec<String> = Vec::new();

                for chunk in fields.chunks(12) {
                    if chunk.len() == 12 {
                        let combined = chunk.join("|");
                        characters_info.push(combined);
                    }
                }

                let mut new_characters: HashMap<i32, Character> = HashMap::new();
                for (index, chunk) in characters_info.iter().enumerate() {
                    debug!("Chunk {}: {}", index + 1, chunk);
                    // now we get index|name|cuid|?|lv|hp|hp_max|mp|mp_max|cid?|?|?    (wihout the last "|")
                    let character = Character::from(chunk);
                    new_characters.insert(character.pos_in_grid, character);
                }
                debug!("------------------");
                if new_characters.len() > 0 {
                    write_shared_characters(&mut state.clone(), new_characters);
                    if let Some(ctx) = state.lock().unwrap().ctx.clone() {
                        debug!("request repaint...");
                        ctx.request_repaint();
                    } else {
                        info!("get_battle_info_thread can't get context, will exit");
                        break;
                    }
                }
            }
        } else {
            // < 0 but not -999?
            debug!("get_battle_info_thread handle={}, IDLE...", tmp_handle);
        }
    }
}

impl MyApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // default addr (2023.08.31)
        let mut addr_name = 0xECB8A8 as i64;
        let mut addr_battle_info = 0x65B5DC as i64;
        let mut addr_battle_flag = 0xD3BEF4 as i64;

        let mut font_path = "c:/Windows/Fonts/msyh.ttc".to_string();

        // try to read settings
        let settings = Config::builder()
            .add_source(config::File::with_name("config.json"))
            .build();
        match settings {
            Ok(s) => {
                if let Ok(v) = s.get_string("memory_addr.name") {
                    if let Ok(i) = i64::from_str_radix(&v, 16) {
                        addr_name = i;
                    }
                }
                if let Ok(v) = s.get_string("memory_addr.battle_info") {
                    if let Ok(i) = i64::from_str_radix(&v, 16) {
                        addr_battle_info = i;
                    }
                }
                if let Ok(v) = s.get_string("memory_addr.battle_flag") {
                    if let Ok(i) = i64::from_str_radix(&v, 16) {
                        addr_battle_flag = i;
                    }
                }
                if let Ok(v) = s.get_string("ui.font") {
                    font_path = v;
                }
            }
            Err(e) => error!("Failed to load config.json: {:?}", e),
        };

        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        setup_custom_fonts(&cc.egui_ctx, font_path);

        let state = Arc::new(Mutex::new(State::default()));

        {
            state.lock().unwrap().handle = 0;
            state.lock().unwrap().characters = HashMap::new();
            state.lock().unwrap().ctx = Some(cc.egui_ctx.clone());
            state.lock().unwrap().addr_name = addr_name;
            state.lock().unwrap().addr_battle_info = addr_battle_info;
            state.lock().unwrap().addr_battle_flag = addr_battle_flag;
        }

        let cloned_state = state.clone();
        std::thread::spawn(move || {
            get_battle_info_thread(cloned_state);
        });

        Self {
            hwnd: "本窗口聚焦时候按F1获取鼠标所在窗口句柄".to_owned(),
            state,
        }
    }
}

impl eframe::App for MyApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let handle = read_shared_handle(&self.state);
        if handle > 0 {
            // clean up
            win32::close_handle(handle);
            write_shared_handle(&mut self.state.clone(), -999); // use to request get_battle_info_thread() return?
        }
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // repaint after
        debug!("update() ...");

        let Self { hwnd, state } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.input(|i| i.key_pressed(egui::Key::F1)) {
                let hwnd_at_cursor = win32::window_at_cursor_point();
                if hwnd_at_cursor > 0 {
                    *hwnd = hwnd_at_cursor.to_string();
                }
            }
            ui.horizontal(|ui| {
                ui.label("窗口：".to_owned());
                ui.text_edit_singleline(hwnd);
                let tmp_handle = read_shared_handle(&state);
                let already_started = tmp_handle > 0;
                if ui
                    .button(if already_started {
                        "看雪中...".to_owned()
                    } else {
                        "开始".to_owned()
                    })
                    .clicked()
                {
                    if already_started {
                        // stop
                        win32::close_handle(tmp_handle);
                        write_shared_handle(&mut state.clone(), 0);
                    } else {
                        if let Ok(hwnd) = hwnd.parse::<i64>() {
                            if hwnd > 0 {
                                let tmp_handle = win32::open_process(hwnd);
                                if tmp_handle > 0 {
                                    // got!
                                    write_shared_handle(&mut state.clone(), tmp_handle);
                                } else {
                                    error!("无法打开进程");
                                    write_shared_handle(&mut state.clone(), -1);
                                }
                            } else {
                                error!("窗口句柄错误");
                            }
                        } else {
                            error!("窗口句柄只能是数字");
                            write_shared_handle(&mut state.clone(), -2);
                        }
                    }
                }
            });

            ui.separator();

            let tmp_handle = read_shared_handle(&state);
            if tmp_handle > 0 {
                ui.horizontal(|ui| {
                    ui.label("玩家名字:".to_owned());
                    let name = win32::read_memory_as_string(
                        tmp_handle,
                        state.lock().unwrap().addr_name,
                        16,
                    );
                    if !name.is_empty() {
                        ui.label(name);
                    } else {
                        ui.label("玩家名字读取失败".to_owned());
                    }
                });

                ui.separator();

                let characters = read_shared_characters(&state);
                let battle_flag = win32::read_memory_as_number(
                    tmp_handle,
                    state.lock().unwrap().addr_battle_flag,
                    4,
                ); // just try
                if characters.is_empty() || battle_flag == 0 {
                    ui.label("战斗未开始".to_owned());
                } else {
                    // battle info grid
                    egui::Grid::new("battle_info_grid")
                        .min_col_width(120.0)
                        .with_row_color(|row, _s| {
                            if row < 2 {
                                Some(egui::Color32::from_rgb(238, 144, 144))
                            } else {
                                Some(egui::Color32::from_rgb(144, 238, 144))
                            }
                        })
                        .show(ui, |ui| {
                            for i in 0..20 {
                                ui.vertical_centered(|ui| {
                                    if characters.contains_key(&i) {
                                        ui.label("".to_owned()); // place holder
                                        let character = characters.get(&i).unwrap();
                                        ui.label(format!("{}", character.name));
                                        ui.label(format!("等级：{}", character.lv));
                                        ui.label(format!(
                                            "生命：{}/{}",
                                            character.hp, character.hp_max
                                        ));
                                        ui.label(format!(
                                            "魔力：{}/{}",
                                            character.mp, character.mp_max
                                        ));
                                        ui.separator();
                                    } else {
                                        // place holder
                                        ui.label("".to_owned());
                                        ui.label("".to_owned());
                                        ui.label("".to_owned());
                                        ui.label("".to_owned());
                                        ui.label("".to_owned());
                                    }
                                });

                                if (i + 1) % 5 == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                }
            } else {
                match tmp_handle {
                    -1 => ui.label("无法打开窗口进程".to_owned()),
                    -2 => ui.label("窗口句柄只能是数字".to_owned()),
                    _ => ui.label("未登录或未绑定".to_owned()),
                };
            }
        });
    }
}
