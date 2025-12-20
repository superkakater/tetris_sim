#[derive(Debug, Clone)]
pub struct Config {
    pub text_only: bool,
    pub seed: Option<u64>,
    pub level: i32,
    pub script_file1: String,
    pub script_file2: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            text_only: false,
            seed: None,
            level: 0,
            script_file1: "tetris_sequence1.txt".to_string(),
            script_file2: "tetris_sequence2.txt".to_string(),
        }
    }
}

pub fn parse_args(args: &[String]) -> Config {
    let mut cfg = Config::default();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-text" => {
                cfg.text_only = true;
            }
            "-seed" => {
                if i + 1 < args.len() {
                    if let Ok(v) = args[i + 1].parse::<u64>() {
                        cfg.seed = Some(v);
                    }
                    i += 1;
                }
            }
            "-scriptfile1" => {
                if i + 1 < args.len() {
                    cfg.script_file1 = args[i + 1].clone();
                    i += 1;
                }
            }
            "-scriptfile2" => {
                if i + 1 < args.len() {
                    cfg.script_file2 = args[i + 1].clone();
                    i += 1;
                }
            }
            "-startlevel" => {
                if i + 1 < args.len() {
                    if let Ok(v) = args[i + 1].parse::<i32>() {
                        cfg.level = v;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    cfg
}
