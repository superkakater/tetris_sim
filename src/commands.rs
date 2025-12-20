use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug)]
pub struct Binding {
    pub is_macro: bool,
    pub single: String,      // canonical command for normal commands/aliases
    pub seq: Vec<String>,    // macro body tokens
}

pub struct CommandTable {
    pub names: Vec<String>,
    pub map: HashMap<String, Binding>,
    pub pending: Vec<String>,            // macro-expanded tokens stack (LIFO)
}

impl CommandTable {
    pub fn new() -> Self {
        let names = vec![
            "left","right","down","cw","ccw","drop",
            "levelup","leveldown","sequence","restart","random","norandom",
            "quit","I","J","L","S","T","O","Z","rename","macro"
        ].into_iter().map(|s| s.to_string()).collect::<Vec<_>>();

        let mut map = HashMap::new();
        for n in &names {
            map.insert(n.clone(), Binding { is_macro: false, single: n.clone(), seq: Vec::new() });
        }

        Self { names, map, pending: Vec::new() }
    }

    pub fn resolve_stem(&self, stem: &str) -> Result<String, String> {
        // exact first
        if self.map.contains_key(stem) {
            return Ok(stem.to_string());
        }

        // unique prefix match among names
        let matches: Vec<&String> = self.names.iter()
            .filter(|name| name.starts_with(stem))
            .collect();

        if matches.is_empty() {
            return Err(format!("Invalid command: {}", stem));
        }
        if matches.len() > 1 {
            return Err(format!("Ambiguous command prefix: {}", stem));
        }
        Ok(matches[0].clone())
    }

    pub fn parse_command_token(&self, token: &str) -> Result<(i32, String), String> {
        if token.is_empty() { return Err("Invalid command".to_string()); }

        // ignore newline sentinel
        if token == "\n" { return Err("".to_string()); }

        let chars: Vec<char> = token.chars().collect();
        let n = chars.len();

        let mut i = 0;
        while i < n && chars[i].is_ascii_digit() { i += 1; }
        let prefix_num: String = chars[..i].iter().collect();

        let mut j: i32 = (n as i32) - 1;
        while j >= 0 && chars[j as usize].is_ascii_digit() { j -= 1; }
        let suffix_num: String = if (j as usize) + 1 < n { chars[(j as usize)+1..].iter().collect() } else { "".to_string() };

        let stem: String = if i <= j as usize { chars[i..=j as usize].iter().collect() } else { "".to_string() };

        if stem.is_empty() && (!prefix_num.is_empty() || !suffix_num.is_empty()) {
            return Err(format!("Invalid command: {}", token));
        }

        let mut repeat = if !prefix_num.is_empty() {
            prefix_num.parse::<i32>().unwrap_or(1)
        } else if !suffix_num.is_empty() {
            suffix_num.parse::<i32>().unwrap_or(1)
        } else { 1 };
        if repeat <= 0 { repeat = 1; }

        let resolved_name = self.resolve_stem(&stem)?;
        let binding = self.map.get(&resolved_name).ok_or_else(|| format!("Invalid command: {}", token))?;

        let command_out = if binding.is_macro {
            resolved_name.clone()
        } else {
            if binding.single.is_empty() { resolved_name.clone() } else { binding.single.clone() }
        };

        let multiplier_not_allowed = matches!(command_out.as_str(),
            "drop" | "restart" | "quit" | "sequence" | "random" | "norandom" | "macro"
        );

        if multiplier_not_allowed && repeat != 1 {
            // mimic C++: ignore multiplier
            repeat = 1;
        }

        Ok((repeat, command_out))
    }

    pub fn define_alias(&mut self, new_name: &str, old_name: &str) -> Result<String, String> {
        if self.map.contains_key(new_name) {
            return Err(format!("rename error: '{}' is already in use", new_name));
        }
        let old = self.map.get(old_name).ok_or_else(|| format!("rename error: '{}' is not a valid base command", old_name))?;
        if old.is_macro {
            return Err(format!("rename error: '{}' is not a valid base command", old_name));
        }
        let canonical = if old.single.is_empty() { old_name.to_string() } else { old.single.clone() };
        self.map.insert(new_name.to_string(), Binding { is_macro: false, single: canonical.clone(), seq: Vec::new() });
        self.names.push(new_name.to_string());
        Ok(format!("Alias created: '{}' → '{}'", new_name, canonical))
    }

    pub fn define_macro(&mut self, name: &str, seq: Vec<String>) -> Result<String, String> {
        if self.map.contains_key(name) {
            return Err(format!("macro error: name '{}' is already in use", name));
        }
        if seq.is_empty() {
            return Err(format!("macro error: empty body for '{}'", name));
        }
        self.map.insert(name.to_string(), Binding { is_macro: true, single: String::new(), seq });
        self.names.push(name.to_string());
        Ok(format!("Macro '{}' defined.", name))
    }

    pub fn is_macro_name(&self, name: &str) -> bool {
        self.map.get(name).map(|b| b.is_macro).unwrap_or(false)
    }

    pub fn macro_seq(&self, name: &str) -> Option<Vec<String>> {
        self.map.get(name).and_then(|b| if b.is_macro { Some(b.seq.clone()) } else { None })
    }
}

// ===== Token sources with newline sentinel =====

pub enum Source {
    Stdin,
    File { tokens: VecDeque<String> },
}

pub struct TokenStream {
    pub sources: Vec<Source>,
    stdin_buf: VecDeque<String>,
}

impl TokenStream {
    pub fn new() -> Self {
        Self { sources: vec![Source::Stdin], stdin_buf: VecDeque::new() }
    }

    pub fn push_file(&mut self, file: &str) -> Result<(), String> {
        let content = std::fs::read_to_string(file)
            .map_err(|_| format!("Cannot open sequence file: {}", file))?;
        let mut tokens = VecDeque::new();
        for line in content.lines() {
            for tok in line.split_whitespace() {
                tokens.push_back(tok.to_string());
            }
            tokens.push_back("\n".to_string());
        }
        self.sources.push(Source::File { tokens });
        Ok(())
    }

    pub fn next_token(&mut self, pending: &mut Vec<String>) -> Option<String> {
        // macro pending tokens first (LIFO)
        if let Some(t) = pending.pop() {
            return Some(t);
        }

        loop {
            let last = self.sources.last_mut()?;
            match last {
                Source::File { tokens } => {
                    if let Some(t) = tokens.pop_front() {
                        return Some(t);
                    } else {
                        self.sources.pop();
                        continue;
                    }
                }
                Source::Stdin => {
                    if let Some(t) = self.stdin_buf.pop_front() {
                        return Some(t);
                    }
                    // read a new line
                    let mut line = String::new();
                    let n = std::io::stdin().read_line(&mut line).ok()?;
                    if n == 0 { return None; } // EOF
                    for tok in line.split_whitespace() {
                        self.stdin_buf.push_back(tok.to_string());
                    }
                    self.stdin_buf.push_back("\n".to_string());
                    continue;
                }
            }
        }
    }
}
