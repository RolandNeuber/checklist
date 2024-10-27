use std::env;
use std::fs;
use std::io::Error;
use regex::Regex;
use once_cell::sync::Lazy;
use std::string::ToString;
use std::cmp;
use const_str;

const DATE_REGEX: Lazy<Regex> = regex_static::lazy_regex!(r"^\d\d\d\d-\d\d-\d\d$");

pub struct Config {
    file_path: String,
    args: Vec<String>,
}

impl Config {
    pub fn build(mut args: Vec<String>) -> Result<Config, &'static str> {
        
        let file_path = match env::var("CHECKLIST_FILE") {
            Ok(var) => var,
            Err(msg) => panic!("{msg}"),
        };

        args = args.drain(2..).collect();

        Ok(Config {
            file_path,
            args,
        })
    }
}

struct TaskEntry {
    task_name: String,
    due_date: String,
    interval: u32,
}

impl TaskEntry {
    fn deserialize(serialization: &str) -> Result<TaskEntry, String> {
        let v: Vec<&str> = serialization.split(',').collect();
        if v.len() != 3 {
            return Err("incorrect number of arguments for deserialization, expected 3".to_string());
        }

        if !DATE_REGEX.is_match(&v[1]) {
            return Err("due date must be a date in the format yyyy-mm-dd".to_string());
        }

        let interval = match v[2].parse::<u32>() {
            Ok(content) => content,
            Err(e) => return Err(e.to_string())
        };

        Ok(TaskEntry {
            task_name: v[0].to_string(),
            due_date: v[1].to_string(),
            interval
        })
    }

    fn build(task_name: String, due_date: String, interval: u32) 
        -> Result<TaskEntry, &'static str> {
        if task_name.contains(',') {
            return Err("task name must not contain commas");
        }

        // let re = Regex::new(r"^\d\d\d\d-\d\d-\d\d$").expect("invalid regex");

        if !DATE_REGEX.is_match(&due_date) {
            return Err("due date must be a date in the format yyyy-mm-dd");
        }

        Ok(TaskEntry {
            task_name,
            due_date,
            interval,
        })
    }

    fn as_table_entry(&self, column_width: [usize; 3]) -> String {
        format!(
            "{:width1$} {:width2$} {:width3$}", 
            &self.task_name, 
            &self.due_date,
            &self.interval, 
            width1=column_width[0], 
            width2=column_width[1], 
            width3=column_width[2]
        )
    }
}

impl ToString for TaskEntry {
    fn to_string(&self) -> String {
        format!(
            "Task name: {}, Due until: {}, Interval: {} days", 
            &self.task_name, 
            &self.due_date, 
            &self.interval
        )
    }
}

fn run(command: Result<fn(config: Config) -> Result<(), &'static str>, &'static str>, config: Config) -> Result<(), &'static str> {
    Ok(())
}

pub fn parse_command(command_str: &str) 
    -> Result<fn(config: Config) -> Result<(), String>, &'static str> {
        
    match command_str {
        "add"       => Ok(add),
        "remove"    => Ok(remove),
        "list"      => Ok(list),
        "check"     => Ok(check),
        "uncheck"   => Ok(uncheck),
        _           => Err("invalid command"),
    }
}

fn add(config: Config) -> Result<(), String> {
    // add     [task_name] [relative_start_date] [interval](optional, once)
    if config.args.len() < 2 {
        return Err("not enough parameters".to_string());
    }
    
    let interval = if config.args.len() < 3 {
        "0"
    } else {
        &config.args[2]
    };

    let res = fs::write(
        config.file_path, 
        format!(
            "{},{},{}",
            config.args[0], 
            config.args[1], 
            interval
        )
    );

    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string())
    }
}

fn remove(config: Config) -> Result<(), String> {
    // remove  [task_name]
    if config.args.len() < 1 {
        return Err("not enough parameters".to_string());
    }
    
    let checklist: Result<String, Error> = fs::read_to_string(&config.file_path);
    let checklist: String = match checklist {
        Ok(content) => content,
        Err(e) => return Err(e.to_string())
    };

    let mut new_checklist = String::new();
    let mut found = false;
    for line in checklist.lines() {
        if !line.starts_with(format!("{}{}", config.args[0], ',').as_str()) {
            new_checklist.push_str(line);
            break;
        }
        else {
            found = true;
        }
    }
    
    if !found {
        return Err(format!("cannot find task named \"{}\"", config.args[0]));
    }

    let res = fs::write(config.file_path, new_checklist);

    Ok(())
}

fn list(config: Config) -> Result<(), String> {
    // list 
    let checklist: String = 
        match fs::read_to_string(&config.file_path) {
            Ok(content) => content,
            Err(e) => return Err(e.to_string())
        };
    
    let mut length: [usize; 3] = [0; 3];
    for line in checklist.lines() {
        let v: Vec<&str> = line.split(',').collect();
        for i in 0..3 {
            length[i] = cmp::max(length[i], v[i].len());
        }
    }   
    for i in 0..3 {
        length[i] = cmp::max(length[i], ["task", "due until", "interval"][i].len());
    }

    println!(
        "{:width1$} {:width2$} {:width3$}", 
        "task", 
        "due until", 
        "interval", 
        width1=length[0], 
        width2=length[1], 
        width3=length[2]
    );
    println!("{}", "-".repeat(length.iter().sum::<usize>() + 2));
    for line in checklist.lines() {
        println!("{}", TaskEntry::deserialize(line)?.as_table_entry(length));
    }

    Ok(())
}

fn check(config: Config) -> Result<(), String> {
    // check   [task_name] 
    if config.args.len() < 1 {
        return Err("not enough parameters".to_string());
    }
    Ok(())
}

fn uncheck(config: Config) -> Result<(), String> {
    // uncheck [task_name]
    if config.args.len() < 1 {
        return Err("not enough parameters".to_string());
    }
    Ok(())
}
