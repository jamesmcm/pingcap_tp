use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Seek, SeekFrom, Write},
    path::PathBuf,
};
use thiserror::Error;
pub struct KvStore {
    map: HashMap<String, u64>,
    log: File,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Command {
    Rm(String),
    Set(String, String),
}

// Need to export for tests
pub type Result<T> = std::result::Result<T, KvError>;

impl KvStore {
    pub fn get(&self, key: String) -> Result<Option<String>> {
        // Read values from log pointers
        let val = match self.map.get(&key) {
            Some(i) => {
                let mut new_f = self.log.try_clone()?;
                new_f.seek(SeekFrom::Start(*i))?;
                let mut br = BufReader::new(new_f);
                let mut l: String = String::new();
                br.read_line(&mut l)?;
                // println!("get serde: {}: {}", i, &l);
                let kv: Command = serde_json::from_str(l.trim())?;
                match kv {
                    Command::Set(_k, v) => Some(v),
                    _ => return Err(KvError::BadLogEntry),
                }
            }
            None => None,
        };
        Ok(val)
    }
    pub fn set(&mut self, key: String, val: String) -> Result<()> {
        // Don't need clone here
        let cmd = Command::Set(key.clone(), val);

        // Write to log
        let j = serde_json::to_string(&cmd)?;
        let index = self.log.stream_position()?;
        // println!("write serde: {}: {}", index, &j);
        writeln!(self.log, "{}", j)?;
        self.log.flush()?;

        // Write in map
        // Write log pointer instead of val
        self.map
            .entry(key)
            .and_modify(|v| *v = index)
            .or_insert(index);

        if index > 500 {
            self.compact()?;
        }
        Ok(())
    }
    pub fn remove(&mut self, key: String) -> Result<()> {
        if !self.map.contains_key(&key) {
            return Err(KvError::KeyNoExist);
        }
        // TODO: Don't need clone here
        let cmd = Command::Rm(key.clone());

        // Write to log
        let j = serde_json::to_string(&cmd)?;
        writeln!(self.log, "{}", j)?;
        self.log.flush()?;

        // Write in map
        self.map.remove(&key);
        Ok(())
    }
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        // Input is directory in tests (wtf?)
        let mut pb: PathBuf = path.into();
        pb.push("kvs.log");
        // println!("open path: {:?}", pb);
        let mut f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(pb.as_path())?;
        let mut map = HashMap::new();

        // Read and replay log
        let mut line = String::new();
        let mut pos: u64 = 0;
        let mut bufreader = BufReader::new(f.try_clone()?);
        while let Ok(num) = bufreader.read_line(&mut line) {
            if num == 0 {
                break;
            }

            // println!("open serde: {}: {}", pos, &line.trim());
            let cmd: Command = serde_json::from_str(line.as_str())?;
            match cmd {
                Command::Set(key, _val) => {
                    map.entry(key).and_modify(|v| *v = pos).or_insert(pos);
                }
                Command::Rm(key) => {
                    map.remove(&key);
                }
            }
            pos += num as u64;
            line.clear();
        }

        f.seek(SeekFrom::End(0))?;
        Ok(Self { map, log: f })
    }
    pub fn compact(&mut self) -> Result<()> {
        let mut latest_entries: HashMap<String, Command> = HashMap::new();

        {
            let mut newf = self.log.try_clone()?;
            newf.seek(SeekFrom::Start(0))?;
            let mut bufreader = BufReader::new(newf);
            let mut line = String::new();
            let mut _pos: u64 = 0;

            while let Ok(num) = bufreader.read_line(&mut line) {
                if num == 0 {
                    break;
                }

                // println!("compact read serde: {}: {}", _pos, &line.trim());
                let cmd: Command = serde_json::from_str(line.as_str())?;
                let key = match &cmd {
                    Command::Rm(k) => k,
                    Command::Set(k, _) => k,
                };

                latest_entries
                    .entry(key.clone())
                    .and_modify(|v| *v = cmd.clone())
                    .or_insert(cmd);
                _pos += num as u64;
                line.clear();
            }
        }

        // Truncate log
        self.log.set_len(0)?;
        self.log.seek(SeekFrom::Start(0))?;

        for (key, cmd) in latest_entries.into_iter() {
            let j = serde_json::to_string(&cmd)?;
            let index = self.log.stream_position()?;
            // println!("write serde: {}: {}", index, &j);
            // println!("compact write serde: {}: {}", index, &j);
            writeln!(self.log, "{}", j)?;
            match &cmd {
                Command::Set(_, _) => {
                    self.map
                        .entry(key)
                        .and_modify(|v| *v = index)
                        .or_insert(index);
                }
                Command::Rm(_) => {
                    self.map.remove(key.as_str());
                }
            }
        }

        self.log.flush()?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum KvError {
    #[error("could not open path")]
    PathError(#[from] std::io::Error),
    #[error("could not serialize to JSON")]
    SerializeError(#[from] serde_json::Error),
    #[error("Key not found")]
    KeyNoExist,
    #[error("Bad log entry")]
    BadLogEntry,
}
