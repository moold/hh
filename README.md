# hh
Quickly save useful bash commands. `hh` is designed to save useful commands you just ran, not aim to save all command records.

## Motivation
As a bioinformatics researcher, I use `Linux Bash` most of my work time. Many projects usually span months or even years, some analytical workflow or details often need to be backtracked when the project is to be completed. So in this case, I have to save some important bash commands. I usually use `vim` to open a file and paste the commands that have just been ran into this file, and this is a very troublesome :anguished: thing, becasue I need to do it again and again. Also, if the saved command has errors or is dupulicated, I have to handled it by manually. `hh` was designed to solve this problem. When you type `hh`, `hh` will automatically save the commands you just ran, and ignore some simple commands (from user settings).

## Features
* Easily and quickly, hit the keyboard twice and finished within <= 10ms.
* Ignore some simple commands, such `ls`, `less` et., al. Any commands with pipe (`|`) or redirect (`>`, `>>`, `1>`, `2>`) will not be ignored.
* Ignore duplicate commands
* Save multiple commands at once
* Undo the last insert operation easily
* Save commands with time and user

## Installation

#### Dependencies

`hh` is written in rust, see [here](https://www.rust-lang.org/tools/install) to install `Rust` first.

#### Download and install

```
git clone https://github.com/moold/hh.git
cd hh && cargo build --release
```

#### Configure 
```
echo "source `pwd`/env/hh.profile" >> $HOME/.bash_profile
```
***Note*** :
* You can refresh your environment variables by `source $HOME/.bash_profile` if you don't want to relogin but want to use `hh` immediately.
* You can change the `HISTIGNORE` and `HHIGNORE` variable in the file `./env/hh.profile` to change the default commands that ignored by `hh`.

## Usage
* Save one command you just run: 
```
hh
```
This will generate a file named `hh.sh` in the current directory.

* Save N (the number you want to save) commands you just run:
```
hh N
```
*  Reset the last insert operation or delete the last inserted content: 
```
hh -r
```
* Only record the fourth record from last: 
```
hh -i 4
```

## Parameters
Use `hh -h` to see options.
