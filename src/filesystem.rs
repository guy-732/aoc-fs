use core::str;
use std::{
    os::unix::ffi::OsStrExt,
    time::{Duration, UNIX_EPOCH},
};

use chrono::{DateTime, Datelike, FixedOffset, Utc};
use fuser::FileAttr;

use crate::config::Config;

// AoC started in 2015, so year 2000 day 0 can be used as a marker for the `latest` symlink at fs root
const LATEST_ROOT_INO: u64 = DayAndYear::new(2000, 0).to_ino();
const AOC_FIRST_YEAR: u32 = 2015;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DayAndYear {
    year: u32,
    day: u8,
}

impl DayAndYear {
    fn last_unlocked_puzzle() -> DayAndYear {
        let now = Utc::now();
        let current_time: DateTime<FixedOffset> = now.with_timezone(
            &FixedOffset::west_opt(3600 * 5)
                .expect("FixedOffset::east_opt(3600 * 5) returned None"),
        );

        let (year, day) = if current_time.month() == 12 {
            let mut day = current_time.day();
            if day > 25 {
                day = 25;
            }

            (current_time.year(), day)
        } else {
            (current_time.year() - 1, 25)
        };

        DayAndYear::new(year as u32, day as u8)
    }

    const fn new(year: u32, day: u8) -> DayAndYear {
        DayAndYear { year, day }
    }

    const fn from_ino(ino: u64) -> DayAndYear {
        DayAndYear {
            year: (ino / 100) as u32,
            day: (ino % 100) as u8,
        }
    }

    const fn to_ino(self) -> u64 {
        (self.year * 100) as u64 + self.day as u64
    }

    const fn file_type(self) -> Result<fuser::FileType, libc::c_int> {
        use fuser::FileType;

        match self.day {
            0 => Ok(FileType::Directory),
            1..=25 => Ok(FileType::RegularFile),
            26 => Ok(FileType::Symlink),
            _ => Err(libc::ENOENT),
        }
    }
}

#[derive(Debug)]
pub struct AoCFilesystem {
    uid: u32,
    gid: u32,
    config: Config,
}

impl AoCFilesystem {
    pub fn new(config: Config) -> Self {
        dbg!(DayAndYear::last_unlocked_puzzle());
        Self {
            uid: unsafe { libc::getuid() },
            gid: unsafe { libc::getgid() },
            config,
        }
    }

    fn getattr_template(&self, ino: u64) -> FileAttr {
        FileAttr {
            ino,
            size: 0,
            blocks: 0,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: fuser::FileType::RegularFile,
            perm: 0o444,
            nlink: 1,
            uid: self.uid,
            gid: self.gid,
            rdev: 0,
            blksize: 0,
            flags: 0,
        }
    }

    fn getattr_impl(&mut self, ino: u64) -> Result<(Duration, FileAttr), libc::c_int> {
        let latest = DayAndYear::last_unlocked_puzzle();
        let day_info = DayAndYear::from_ino(ino);
        if day_info.year < AOC_FIRST_YEAR || day_info.year > latest.year {
            if ino == LATEST_ROOT_INO {
                let mut attr = self.getattr_template(ino);
                attr.kind = fuser::FileType::Symlink;
                return Ok((Duration::from_secs(1), attr));
            }

            return Err(libc::ENOENT);
        }

        if day_info.year == latest.year && day_info.day <= 25 {
            if day_info.day > latest.day {
                return Err(libc::ENOENT);
            }
        }

        let mut attr = self.getattr_template(ino);
        match day_info.file_type()? {
            fuser::FileType::RegularFile => {
                attr.blksize = 4096;
                attr.size = 4096;
                attr.blocks = 1;
            }
            fuser::FileType::Directory => {
                attr.kind = fuser::FileType::Directory;
                attr.perm = 0o555;
                attr.nlink = 2;
            }
            fuser::FileType::Symlink => {
                attr.kind = fuser::FileType::Symlink;
                attr.perm = 0o777;
            }
            _ => unreachable!("File type was neither Directory, RegularFile nor Symlink"),
        }

        Ok((Duration::from_secs(1), attr))
    }

    fn lookup_year(&self, year: u32, name: &str) -> Result<u64, libc::c_int> {
        let latest = DayAndYear::last_unlocked_puzzle();
        if year < AOC_FIRST_YEAR || year > latest.year {
            return Err(libc::ENOENT);
        }

        let name = name.trim_end_matches(".txt").trim_end_matches(".input");
        if name == "latest" {
            return Ok(DayAndYear::new(year, 26).to_ino());
        }

        let name = name.trim_start_matches("day").trim_start_matches('0');
        let day = match name.parse::<u8>() {
            Ok(day) => day,
            Err(_) => return Err(libc::ENOENT),
        };

        if day < 1 || day > 25 {
            return Err(libc::ENOENT);
        }

        Ok(DayAndYear::new(year, day).to_ino())
    }

    fn lookup_root(&self, name: &str) -> Result<u64, libc::c_int> {
        if name == "latest" {
            return Ok(LATEST_ROOT_INO);
        }

        let year = match name.parse::<u32>() {
            Ok(res) => res,
            Err(_) => return Err(libc::ENOENT),
        };

        let latest = DayAndYear::last_unlocked_puzzle();
        if year < AOC_FIRST_YEAR || year > latest.year {
            Err(libc::ENOENT)
        } else {
            Ok(DayAndYear::new(year, 0).to_ino())
        }
    }

    fn readlink_impl(&self, ino: u64) -> Result<String, libc::c_int> {
        if ino == LATEST_ROOT_INO {
            let latest = DayAndYear::last_unlocked_puzzle();
            return Ok(latest.year.to_string());
        }

        let year = (ino / 100) as u32;
        let day = ino % 100;
        if day != 26 {
            return Err(libc::EINVAL);
        }

        let latest = DayAndYear::last_unlocked_puzzle();
        if year < AOC_FIRST_YEAR || year > latest.year {
            return Err(libc::ENOENT);
        }

        let day = if latest.year == year { latest.day } else { 25 };

        return Ok(format!("day{day:02}.txt"));
    }
}

impl fuser::Filesystem for AoCFilesystem {
    fn init(
        &mut self,
        _req: &fuser::Request<'_>,
        _config: &mut fuser::KernelConfig,
    ) -> Result<(), libc::c_int> {
        log::trace!("Filesystem mounted");
        Ok(())
    }

    fn destroy(&mut self) {
        log::trace!("Filesystem unmounted, destroy() called");
    }

    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) {
        let name = match str::from_utf8(name.as_bytes()) {
            Ok(s) => s,
            Err(_) => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        log::trace!("lookup(..., parent={parent}, name={name:?})");
        let ino = if parent == fuser::FUSE_ROOT_ID {
            match self.lookup_root(name) {
                Ok(ino) => ino,
                Err(e) => {
                    reply.error(e);
                    return;
                }
            }
        } else {
            let year = parent / 100;
            if parent % 100 != 0 || parent == LATEST_ROOT_INO {
                reply.error(libc::ENOTDIR);
                return;
            }

            match self.lookup_year(year as u32, name) {
                Ok(ino) => ino,
                Err(e) => {
                    reply.error(e);
                    return;
                }
            }
        };

        let (ttl, attr) = match self.getattr_impl(ino) {
            Ok(res) => res,
            Err(e) => {
                reply.error(e);
                return;
            }
        };

        reply.entry(&ttl, &attr, 0);
    }

    fn getattr(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: Option<u64>,
        reply: fuser::ReplyAttr,
    ) {
        log::trace!("getattr(..., ino={ino}, fh={fh:?})");

        match ino {
            fuser::FUSE_ROOT_ID => {
                let latest = DayAndYear::last_unlocked_puzzle();
                let mut attr = self.getattr_template(fuser::FUSE_ROOT_ID);
                attr.kind = fuser::FileType::Directory;
                attr.perm = 0o555;
                attr.nlink = 2 + (latest.year - AOC_FIRST_YEAR);

                reply.attr(&Duration::from_secs(1), &attr);
            }
            other => match self.getattr_impl(other) {
                Ok((ttl, attr)) => reply.attr(&ttl, &attr),
                Err(errno_val) => {
                    log::warn!("getattr received a request for ino {other}, error occurred (errno = {errno_val})");
                    reply.error(errno_val);
                }
            },
        }
    }

    fn readlink(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyData) {
        match self.readlink_impl(ino) {
            Ok(link) => {
                log::trace!("readlink(..., ino={}) => {:?}", ino, &link);
                reply.data(link.as_bytes());
            }
            Err(err) => {
                log::trace!("readlink(..., ino={}) => error (errno={})", ino, err);
                reply.error(err);
            }
        }
    }

    fn readdir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: fuser::ReplyDirectory,
    ) {
        log::trace!("readdir(..., ino={ino}, offset={offset})");

        let latest = DayAndYear::last_unlocked_puzzle();
        if ino == fuser::FUSE_ROOT_ID {
            if offset == 0 {
                if reply.add(ino, 1, fuser::FileType::Directory, ".") {
                    reply.ok();
                    return;
                }
            }

            if offset <= 1 {
                if reply.add(fuser::FUSE_ROOT_ID, 2, fuser::FileType::Directory, "..") {
                    reply.ok();
                    return;
                }
            }

            let offset2 = if offset >= 2 {
                (offset - 2) as usize
            } else {
                0
            };

            for (i, year) in (AOC_FIRST_YEAR..=latest.year).enumerate().skip(offset2) {
                let date = DayAndYear::new(year, 0);
                if reply.add(
                    date.to_ino(),
                    (i + 3) as i64,
                    fuser::FileType::Directory,
                    format!("{year}").as_str(),
                ) {
                    reply.ok();
                    return;
                }
            }

            if offset <= (latest.year - AOC_FIRST_YEAR + 3) as i64 {
                let _ = reply.add(
                    LATEST_ROOT_INO,
                    ((latest.year - AOC_FIRST_YEAR) + 4) as i64,
                    fuser::FileType::Symlink,
                    "latest",
                );
            }

            reply.ok();
            return;
        }

        let year = (ino / 100) as u32;
        if ino % 100 != 0 {
            reply.error(libc::ENOTDIR);
            return;
        }

        if year < AOC_FIRST_YEAR || year > latest.year {
            reply.error(libc::ENOENT);
            return;
        }

        if offset == 0 {
            if reply.add(ino, 1, fuser::FileType::Directory, ".") {
                reply.ok();
                return;
            }
        }

        if offset <= 1 {
            if reply.add(fuser::FUSE_ROOT_ID, 2, fuser::FileType::Directory, "..") {
                reply.ok();
                return;
            }
        }

        let offset2 = if offset >= 2 {
            (offset - 2) as usize
        } else {
            0
        };

        let max_day = if year == latest.year { latest.day } else { 25 };

        for i in (1..=max_day).skip(offset2) {
            if reply.add(
                DayAndYear::new(year, i).to_ino(),
                (i + 2) as i64,
                fuser::FileType::RegularFile,
                format!("day{i:02}.txt").as_str(),
            ) {
                reply.ok();
                return;
            }
        }

        if offset <= max_day as i64 + 3 {
            let _ = reply.add(
                DayAndYear::new(year, 26).to_ino(),
                max_day as i64 + 4,
                fuser::FileType::Symlink,
                "latest",
            );
        }

        reply.ok();
    }
}