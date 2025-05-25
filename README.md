# Checklist command

The checklist command is a simple CLI to organize your TODOs.

# Command Specification

<pre>
[CHECKLIST_FILE=](optional) checklist
  add     [task_name] [due_date] [interval](optional, once)
  remove  [task_name]
  list
  check   [task_name]
  uncheck [task_name]
</pre>

The `task_name` argument may contain any characters besides commas. This limitation will probably be removed in the future. \
The `due_date` needs to be supplied in the format `%Y-%m-%d`, e.g. `2000-04-30`. Other formats may be supported in the future. \
The `interval` need to be a positive integer, the number of days until a task repeats or the literal `once`, which is also the default value.

# Checklist file path

A checklist file can be provided by either
* calling the checklist command with the env variable `CHECKLIST_FILE` set to the location of the file
* setting the env variable `CHECKLIST_FILE` in your environment

Currently it is an error to have no path specified. This is subject to change an will probably be changed to include a default path in the future.

# Checklist file structure

The file that saves the current state of the checklist is a CSV following this structure:

<pre>
task_name,due_date,interval
---task 1---
---task 2---
...
---task n---
</pre>
