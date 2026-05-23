use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn read_modified_time(path: &Path) -> Option<SystemTime> {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
}

pub fn restore_modified_time(path: &Path, modified_at: Option<SystemTime>) -> Result<(), String> {
    let Some(modified_at) = modified_at else {
        return Ok(());
    };
    let file = fs::File::open(path)
        .map_err(|error| format!("打开文件以恢复修改时间失败 ({}): {}", path.display(), error))?;
    file.set_modified(modified_at)
        .map_err(|error| format!("恢复文件修改时间失败 ({}): {}", path.display(), error))
}

pub fn system_time_from_unix_millis(timestamp_ms: i128) -> Option<SystemTime> {
    if timestamp_ms < 0 || timestamp_ms > u64::MAX as i128 {
        return None;
    }
    UNIX_EPOCH.checked_add(Duration::from_millis(timestamp_ms as u64))
}

pub fn same_modified_time_millis(left: Option<SystemTime>, right: Option<SystemTime>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => {
            let left = left
                .duration_since(UNIX_EPOCH)
                .ok()
                .map(|value| value.as_millis());
            let right = right
                .duration_since(UNIX_EPOCH)
                .ok()
                .map(|value| value.as_millis());
            left == right
        }
        (None, None) => true,
        _ => false,
    }
}
