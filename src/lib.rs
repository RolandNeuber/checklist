use chrono::{Duration, Local, NaiveDate};
use colored::Colorize;
use directories_next::ProjectDirs;
use std::cmp;
use std::env;
use std::fmt::Display;
use std::fs;
use std::fs::OpenOptions;
use std::io::Error;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::string::ToString;

pub struct Config {
    file_path: PathBuf,
    args: Vec<String>,
}

impl Config {
    /// # Errors
    ///
    /// Returns an error when the env var `CHECKLIST_FILE` is not set and a project directory could not be generated automatically.
    /// Returns an error when the determined checklist file could not be touched (e.g. generated if it did not exist).
    pub fn build(mut args: Vec<String>) -> Result<Self, String> {
        let file_path;
        if let Ok(path) = env::var("CHECKLIST_FILE") {
            file_path = PathBuf::from(&path);
        } else {
            file_path = ProjectDirs::from("", "", "Checklist")
            .ok_or_else(|| "Could not generate project directory path, consider manually specifying a path using the CHECKLIST_FILE env variable.".to_owned())?
            .data_dir()
            .to_path_buf();
        }

        match OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&file_path)
        {
            Ok(_) => (),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => (),
            Err(msg) => return Err(msg.to_string()),
        }

        args = args.drain(2..).collect();

        Ok(Self { file_path, args })
    }
}

struct TaskEntry {
    task_name: String,
    due_date: NaiveDate,
    interval: u32,
}

impl TaskEntry {
    fn serialize(&self) -> String {
        format!(
            "{},{},{}",
            &self.task_name,
            &self.due_date,
            if self.interval == 0 {
                &0
            } else {
                &self.interval
            }
        )
    }

    fn deserialize(serialization: &str) -> Result<Self, String> {
        let v: Vec<&str> = serialization.split(',').collect();
        if v.len() != 3 {
            return Err(
                "incorrect number of arguments for deserialization, expected 3".to_string(),
            );
        }

        let due_date = match NaiveDate::parse_from_str(v[1], "%Y-%m-%d") {
            Ok(date) => date,
            Err(e) => return Err(e.to_string()),
        };

        let interval = match v[2].parse::<u32>() {
            Ok(content) => content,
            Err(e) => return Err(e.to_string()),
        };

        Ok(Self {
            task_name: v[0].to_string(),
            due_date,
            interval,
        })
    }

    #[allow(dead_code)]
    fn build(task_name: String, due_date: &str, interval: u32) -> Result<Self, String> {
        if task_name.contains(',') {
            return Err("task name must not contain commas".to_string());
        }

        let due_date = match NaiveDate::parse_from_str(due_date, "%Y-%m-%d") {
            Ok(date) => date,
            Err(e) => return Err(e.to_string()),
        };

        Ok(Self {
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
            if self.interval == 0 {
                "once".to_string()
            } else {
                format!("{}", &self.interval)
            },
            width1 = column_width[0],
            width2 = column_width[1],
            width3 = column_width[2]
        )
    }
}

impl Display for TaskEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Task name: {}, Due until: {}, Interval: {} days",
            &self.task_name, &self.due_date, &self.interval
        )
    }
}

#[allow(dead_code)]
struct TaskTable {
    tasks: Vec<TaskEntry>,
}

impl TaskTable {
    #[allow(dead_code)]
    fn serialize(&self) -> String {
        let mut length: [usize; 3] = [0; 3];
        for entry in &self.tasks {
            length[0] = cmp::max(length[0], entry.task_name.len());
            length[1] = cmp::max(length[1], entry.due_date.to_string().len());
            length[2] = cmp::max(length[2], entry.interval.to_string().len());
        }

        let mut serialization = String::new();
        for task in &self.tasks {
            serialization = format!("{}\n{}", serialization, task.as_table_entry(length));
        }

        serialization
    }

    #[allow(dead_code)]
    fn deserialize(serialization: &str) -> Result<Self, String> {
        let mut tasks = vec![];
        for line in serialization.lines() {
            tasks.push(TaskEntry::deserialize(line)?);
        }

        Ok(Self { tasks })
    }
}

type Command = fn(config: &mut Config) -> Result<(), String>;

/// # Errors
///
/// Returns an error when an unknown command is supplied.
pub fn parse_command(command_str: &str) -> Result<Command, &'static str> {
    match command_str {
        "add" => Ok(add),
        "remove" => Ok(remove),
        "list" => Ok(list),
        "check" => Ok(check),
        "uncheck" => Ok(uncheck),
        _ => Err("invalid command"),
    }
}

fn add(config: &mut Config) -> Result<(), String> {
    // add     [task_name] [relative_start_date] [interval](optional, once)

    if config.args.len() < 2 {
        return Err("not enough parameters".to_string());
    }

    let checklist: Result<String, Error> = fs::read_to_string(&config.file_path);
    let checklist: String = match checklist {
        Ok(content) => content,
        Err(e) => return Err(e.to_string()),
    };

    let mut found = false;
    for line in checklist.lines() {
        if line.starts_with(format!("{}{}", config.args[0], ',').as_str()) {
            found = true;
        }
    }

    if found {
        return Err(format!("entry with name {} already exists", config.args[0]));
    }

    let interval = if config.args.len() < 3 {
        "0"
    } else {
        &config.args[2]
    };

    let entry = TaskEntry::deserialize(
        format!(
            "{},{},{}",
            &config.args[0],
            &config.args[1],
            if interval == "once" { "0" } else { interval }
        )
        .as_str(),
    )?;

    match fs::write(
        config.file_path.clone(),
        format!("{}\n{}", entry.serialize(), checklist),
    ) {
        Ok(()) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

fn remove(config: &mut Config) -> Result<(), String> {
    // remove  [task_name]
    if config.args.is_empty() {
        return Err("not enough parameters".to_string());
    }

    let checklist: Result<String, Error> = fs::read_to_string(&config.file_path);
    let checklist: String = match checklist {
        Ok(content) => content,
        Err(e) => return Err(e.to_string()),
    };

    let mut new_checklist = String::new();
    let mut found = false;
    let mut first_line = true;
    for line in checklist.lines() {
        if line.starts_with(format!("{}{}", config.args[0], ',').as_str()) {
            found = true;
        } else {
            if !first_line {
                new_checklist.push('\n');
            }
            new_checklist.push_str(line);
            first_line = false;
        }
    }

    if !found {
        return Err(format!("cannot find task named \"{}\"", config.args[0]));
    }

    if let Err(e) = fs::write(config.file_path.clone(), new_checklist) {
        return Err(e.to_string());
    }

    Ok(())
}

fn list(config: &mut Config) -> Result<(), String> {
    // list
    let checklist: String = match fs::read_to_string(&config.file_path) {
        Ok(content) => content,
        Err(e) => return Err(e.to_string()),
    };

    let mut lengths: [usize; 3] = [0; 3];
    for line in checklist.lines() {
        let v: Vec<&str> = line.split(',').collect();
        for i in 0..3 {
            lengths[i] = cmp::max(lengths[i], v[i].len());
        }
    }
    for (i, length) in lengths.iter_mut().enumerate() {
        *length = *cmp::max(
            &mut *length,
            &mut ["task", "due until", "interval"][i].len(),
        );
    }

    println!(
        "{:width1$} {:width2$} {:width3$}",
        "task",
        "due until",
        "interval",
        width1 = lengths[0],
        width2 = lengths[1],
        width3 = lengths[2]
    );
    println!("{}", "-".repeat(lengths.iter().sum::<usize>() + 2));
    let now = Local::now().date_naive();
    for line in checklist.lines() {
        let entry = TaskEntry::deserialize(line)?;
        if entry.due_date < now {
            println!("{}", entry.as_table_entry(lengths).red().bold());
        } else {
            println!("{}", entry.as_table_entry(lengths));
        }
    }

    Ok(())
}

fn check(config: &mut Config) -> Result<(), String> {
    // check   [task_name]
    if config.args.is_empty() {
        return Err("not enough parameters".to_string());
    }

    let checklist: Result<String, Error> = fs::read_to_string(&config.file_path);
    let checklist: String = match checklist {
        Ok(content) => content,
        Err(e) => return Err(e.to_string()),
    };

    let mut found = false;
    let mut entry = TaskEntry {
        task_name: config.args[0].clone(),
        due_date: Local::now().naive_local().into(),
        interval: 0,
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

    remove(config)?;

    if entry.interval == 0 {
        return Ok(());
    }

    let today: NaiveDate = Local::now().naive_local().into();
    let Some(new_due_date) = today.checked_add_signed(Duration::days(entry.interval.into())) else {
        return Err("could not calculate new due date".to_string());
    };

    add(&mut Config {
        file_path: config.file_path.clone(),
        args: vec![
            config.args[0].clone(),     // task_name
            new_due_date.to_string(),   // due_date
            entry.interval.to_string(), // interval
        ],
    })?;

    Ok(())
}

fn uncheck(config: &mut Config) -> Result<(), String> {
    // uncheck [task_name]
    if config.args.is_empty() {
        return Err("not enough parameters".to_string());
    }
    Ok(())
}
