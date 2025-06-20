# Advent of Code Filesystem
This mounts your advent of code inputs as a FUSE filesystem.

## Filesystem when inspected on the 2nd of December 2024
```
.
├── 2015
│   ├── day01.txt
│   ├── day02.txt
│   ├── ...
│   ├── day25.txt
│   └── latest -> day25.txt
├── 2016
│   ├── day01.txt
│   ├── day02.txt
│   ├── ...
│   ├── day25.txt
│   └── latest -> day25.txt
├── 2017
│   ├── day01.txt
│   ├── day02.txt
│   ├── ...
│   ├── day25.txt
│   └── latest -> day25.txt
├── 2018
│   ├── day01.txt
│   ├── day02.txt
│   ├── ...
│   ├── day25.txt
│   └── latest -> day25.txt
├── 2019
│   ├── day01.txt
│   ├── day02.txt
│   ├── ...
│   ├── day25.txt
│   └── latest -> day25.txt
├── 2020
│   ├── day01.txt
│   ├── day02.txt
│   ├── ...
│   ├── day25.txt
│   └── latest -> day25.txt
├── 2021
│   ├── day01.txt
│   ├── day02.txt
│   ├── ...
│   ├── day25.txt
│   └── latest -> day25.txt
├── 2022
│   ├── day01.txt
│   ├── day02.txt
│   ├── ...
│   ├── day25.txt
│   └── latest -> day25.txt
├── 2023
│   ├── day01.txt
│   ├── day02.txt
│   ├── ...
│   ├── day25.txt
│   └── latest -> day25.txt
├── 2024
│   ├── day01.txt
│   ├── day02.txt
│   └── latest -> day02.txt
└── latest -> 2024

11 directories, 237 files
```

# Funny side effect of spamming `.trim()` in code
The names listed in through ls on the directories are not all there is, each input files have an infinite
number of names.
```
$ ls -i 2024/{day,}{0{000,},}2{.txt,.input{,.txt{,.txt}},}
202402 2024/00002                202402 2024/2                       202402 2024/day02
202402 2024/00002.input          202402 2024/2.input                 202402 2024/day02.input
202402 2024/00002.input.txt      202402 2024/2.input.txt             202402 2024/day02.input.txt
202402 2024/00002.input.txt.txt  202402 2024/2.input.txt.txt         202402 2024/day02.input.txt.txt
202402 2024/00002.txt            202402 2024/2.txt                   202402 2024/day02.txt
202402 2024/02                   202402 2024/day00002                202402 2024/day2
202402 2024/02.input             202402 2024/day00002.input          202402 2024/day2.input
202402 2024/02.input.txt         202402 2024/day00002.input.txt      202402 2024/day2.input.txt
202402 2024/02.input.txt.txt     202402 2024/day00002.input.txt.txt  202402 2024/day2.input.txt.txt
202402 2024/02.txt               202402 2024/day00002.txt            202402 2024/day2.txt
```
As long as the file matches the following regex, it probably exists (if that day's puzzle is released):
- `/(?:day)*0*([1-9]|1[0-9]|2[0-5])(?:\.input)*(?:\.txt)*/`

Also the latest symlink can match the following for the most recent puzzle of that year:
- `/latest(?:\.input)*(?:\.txt)*/`

As a result, the number of hard links to those files reported is technically wrong, but only 1 of those
links is shown when listing directories anyway.
