use ini::Ini;
use std::{fs, io::Write};

pub enum ProcessorMode {
    Chip8,
    SuperChip,
    XOChip,
}

#[derive(Clone)]
pub struct Palette {
    name: String,
    on_color: u32,
    off_color: u32,
}

pub struct Config {
    pub mode: ProcessorMode,
    pub on_color: u32,
    pub off_color: u32,
    palettes: Vec<Palette>,
}

impl Config {
    pub fn build() -> std::io::Result<Config> {
        let mut conf: Config = Config {
            mode: ProcessorMode::Chip8,
            on_color: 0x00FF00,
            off_color: 0x0,
            palettes: vec![
                Palette {
                    name: String::from("Matrix"),
                    on_color: 0x00FF00,
                    off_color: 0x0,
                },
                Palette {
                    name: String::from("Neon"),
                    on_color: 0x00FFEC,
                    off_color: 0xD600FF,
                },
                Palette {
                    name: String::from("Chill"),
                    on_color: 0xF0F6F0,
                    off_color: 0x222323,
                },
            ],
        };

        let palettes_result = fs::read_to_string("colors.txt");
        let mut palettes = Vec::new();
        match palettes_result {
            Ok(contents) => {
                for line in contents.split("\n") {
                    let mut values = line.split(" ");
                    let name = values.next();
                    if name.is_none() {
                        continue;
                    }
                    let on_color = values.next();
                    if on_color.is_none() {
                        continue;
                    }
                    let off_color = values.next();
                    if off_color.is_none() {
                        continue;
                    }

                    palettes.push(Palette {
                        name: String::from(name.unwrap()),
                        on_color: match u32::from_str_radix(&on_color.unwrap().replace("0x", ""), 16) {
                            Ok(val) => val,
                            Err(e) => {
                                println!("ERROR Failed to parse on_color for pallette {}: {}", name.unwrap(), e);
                                continue;
                            }
                        },
                        off_color: match u32::from_str_radix(&off_color.unwrap().replace("0x", ""), 16) {
                            Ok(val) => val,
                            Err(e) => {
                                println!("ERROR Failed to parse off_color for pallette {}: {}", name.unwrap(), e);
                                continue;
                            }
                        },
                    });
                }
            }
            _ => {
                palettes = conf.palettes;
                let mut file = fs::File::create("colors.txt")?;
                for palette in &palettes {
                    let line = format!(
                        "{} {:#02X} {:#02X}\n",
                        palette.name, palette.on_color, palette.off_color
                    );
                    file.write_all(line.as_bytes())?;
                }
            }
        }

        let ini_result = Ini::load_from_file("config.ini");
        match ini_result {
            Ok(ini) => {
                for (_sec, prop) in &ini {
                    for (key, value) in prop.iter() {
                        match key {
                            "mode" => {
                                conf.mode = match value {
                                    "chip-8" => ProcessorMode::Chip8,
                                    "superchip" => ProcessorMode::SuperChip,
                                    "xochip" => ProcessorMode::XOChip,
                                    _ => ProcessorMode::Chip8,
                                };
                            }
                            "on_color" => conf.on_color = u32::from_str_radix(&value.replace("0x", ""), 16).unwrap(),
                            "off_color" => conf.off_color = u32::from_str_radix(&value.replace("0x", ""), 16).unwrap(),
                            _ => {}
                        }
                    }
                }
            }
            Err(..) => {
                let mut ini = Ini::new();
                ini.with_section(Some("General")).set("mode", "chip-8");
                ini.with_section(Some("Colors"))
                    .set("on_color", format!("{:#04x}", conf.on_color))
                    .set("off_color", format!("{:#04x}", conf.off_color));
                ini.write_to_file("config.ini")?;
            }
        };

        Ok(Config {
            palettes,
            ..conf
        })
    }

    pub fn save_to_file(&self) -> std::io::Result<()>{
        let mut ini = Ini::new();
        ini.with_section(Some("General")).set("mode", match self.mode {
            ProcessorMode::Chip8 => "chip-8",
            ProcessorMode::SuperChip => "superchip",
            ProcessorMode::XOChip => "xochip"
        });
        ini.with_section(Some("Colors"))
            .set("on_color", format!("{:#04x}", self.on_color))
            .set("off_color", format!("{:#04x}", self.off_color));
        ini.write_to_file("config.ini")?;
        Ok(())
    }

    pub fn get_palettes(&self) -> &Vec<Palette> {
        &self.palettes
    }
}

impl Palette {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_colors(&self) -> (u32, u32) {
        (self.on_color, self.off_color)
    }
}
