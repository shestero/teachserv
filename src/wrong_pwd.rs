use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// Last wrong password moment
lazy_static::lazy_static! {
    static ref last_wrong_pwd_time: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
}

pub fn update_wrong_pwd_timestamp() -> Result<(), Box<dyn Error>> {
    let lock_handle = Arc::clone(&last_wrong_pwd_time);
    // Блокируем Mutex
    let mut time_guard = lock_handle.lock().map_err(|e| format!("Mutex poisoned: {}", e))?;

    // Обновляем значение внутри Mutex на текущий момент, оборачивая его в Some
    *time_guard = Some(Instant::now());
    println!("-> Штамп времени обновлен. {:?}", time_guard);
    Ok(())
}

pub fn time_since_last_wrong_pwd() -> Result<Option<Duration>, Box<dyn Error>> {
    let lock_handle = Arc::clone(&last_wrong_pwd_time);
    // Блокируем Mutex
    let time_guard = lock_handle.lock().map_err(|e| format!("Mutex poisoned: {}", e))?;

    Ok( (*time_guard).map(|last_time| last_time.elapsed()) )
}
