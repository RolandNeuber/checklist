use std::env;
use std::fs;
use std::io::Error;
use std::string::ToString;
use std::cmp;
use chrono::{Local, NaiveDate, Duration};
use colored::Colorize;

#[derive(Clone)]
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
    due_date: NaiveDate,
    interval: u32,
}

impl TaskEntry {
    fn serialize(&self) -> String {
        format!("{},{},{}", &self.task_name, &self.due_date, if self.interval == 0 {&0} else {&self.interval})
    }

    fn deserialize(serialization: &str) -> Result<TaskEntry, String> {
        let v: Vec<&str> = serialization.split(',').collect();
        if v.len() != 3 {
            return Err("incorrect number of arguments for deserialization, expected 3".to_string());
        }

        let due_date = match NaiveDate::parse_from_str(&v[1], "%Y-%m-%d") {
            Ok(date) => date,
            Err(e) => return Err(e.to_string())
        };

        let interval = match v[2].parse::<u32>() {
            Ok(content) => content,
            Err(e) => return Err(e.to_string())
        };

        Ok(TaskEntry {
            task_name: v[0].to_string(),
            due_date,
            interval
        })
    }
    
    #[warn(dead_code)]
    fn build(task_name: String, due_date: String, interval: u32) 
        -> Result<TaskEntry, String> {
        if task_name.contains(',') {
            return Err("task name must not contain commas".to_string());
        }

        let due_date = match NaiveDate::parse_from_str(&due_date, "%Y-%m-%d") {
            Ok(date) => date,
            Err(e) => return Err(e.to_string())
        };

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
            if self.interval == 0 {"once".to_string()} else {format!("{}", &self.interval)}, 
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

    let checklist: Result<String, Error> = fs::read_to_string(&config.file_path);
    let checklist: String = match checklist {
        Ok(content) => content,
        Err(e) => return Err(e.to_string())
    };

    let mut found = false;
    for line in checklist.lines() {
        if line.starts_with(format!("{}{}", config.args[0], ',').as_str()) {
            found = true;
        }
    }

    if found {
        return Err(format!("entry with name {} already exists", config.args[0]))
    }
    
    let interval = if config.args.len() < 3 {
        "0"
    } else {
        &config.args[2]
    };
    
    let entry = TaskEntry::deserialize(format!("{},{},{}", &config.args[0], &config.args[1], if interval == "once" {"0"} else {interval}).as_str())?;

    match fs::write(config.file_path, format!("{}\n{}", entry.serialize(), checklist)) {
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
    let mut first_line = true;
    for line in checklist.lines() {
        if !line.starts_with(format!("{}{}", config.args[0], ',').as_str()) {
            if !first_line {
                new_checklist.push_str("\n");
            }
            new_checklist.push_str(line);
            first_line = false;
        }
        else {
            found = true;
        }
    }
    
    if !found {
        return Err(format!("cannot find task named \"{}\"", config.args[0]));
    }

    if let Err(e) = fs::write(config.file_path, new_checklist) {
        return Err(e.to_string());
    };

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
    let now = Local::now().date_naive();
    for line in checklist.lines() {
        let entry = TaskEntry::deserialize(line)?;
        if entry.due_date < now {
            println!("{}", entry.as_table_entry(length).red().bold());
        }
        else {
            println!("{}", entry.as_table_entry(length));
        }
    }

    Ok(())
}

fn check(config: Config) -> Result<(), String> {
    // check   [task_name] 
    if config.args.len() < 1 {
        return Err("not enough parameters".to_string());
    }

    let checklist: Result<String, Error> = fs::read_to_string(&config.file_path);
    let checklist: String = match checklist {
        Ok(content) => content,
        Err(e) => return Err(e.to_string())
    };

    let mut found = false;
    let mut entry = TaskEntry { 
        task_name: config.args[0].clone(), 
        due_date: Local::now().naive_local().into(), 
        interval: 0 
    }; // defaults, so compiler does not complain
    
    for line in checklist.lines() {
        if line.starts_with(format!("{}{}", config.args[0], ',').as_str()) {
            found = true;
            entry = TaskEntry::deserialize(line)?;
        }
    }

    if !found {
        return Err(format!("cannot find task named \"{}\"", config.args[0]));
    }

    remove(config.clone())?;

    if entry.interval == 0 {
        return Ok(());
    }

    let today: NaiveDate = Local::now().naive_local().into();
    let new_due_date = match today.checked_add_signed(Duration::days(entry.interval.into())) {
        Some(content) => content,
        None => return Err("could not calculate new due date".to_string())
    };

    add(Config {
        file_path: config.file_path,
        args: vec!(
            config.args[0].clone(), // task_name
            new_due_date.to_string(), // due_date
            entry.interval.to_string(), // interval
        ),
    })?;

    Ok(())
}

fn uncheck(config: Config) -> Result<(), String> {
    // uncheck [task_name]
    if config.args.len() < 1 {
        return Err("not enough parameters".to_string());
    }
    Ok(())
}
