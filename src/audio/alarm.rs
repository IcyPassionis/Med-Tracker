use rodio::Source;
use std::path::Path;
use std::sync::Mutex;

static STOP_SENDER: Mutex<Option<std::sync::mpsc::Sender<()>>> = Mutex::new(None);

pub fn is_decodable_sound_file(path: &Path) -> bool {
    let Ok(file) = std::fs::File::open(path) else {
        return false;
    };

    rodio::Decoder::new(std::io::BufReader::new(file)).is_ok()
}

pub fn play_alarm(sound_path: &str, volume: f32) {
    stop_alarm();
    let asset_path = sound_path.to_string();
    if !Path::new(&asset_path).exists() {
        eprintln!("Alarm sound not found: {}", asset_path);
        return;
    }
    let volume = volume.clamp(0.0, 1.0);
    let (tx, rx) = std::sync::mpsc::channel();
    *STOP_SENDER.lock().unwrap() = Some(tx);
    std::thread::spawn(move || {
        let Ok((_stream, stream_handle)) = rodio::OutputStream::try_default() else {
            eprintln!("Failed to open audio output");
            return;
        };
        let Ok(sink) = rodio::Sink::try_new(&stream_handle) else {
            eprintln!("Failed to create audio sink");
            return;
        };
        let Ok(file) = std::fs::File::open(&asset_path) else {
            return;
        };
        let Ok(source) = rodio::Decoder::new(std::io::BufReader::new(file)) else {
            eprintln!("Failed to decode alarm sound");
            return;
        };
        sink.set_volume(volume);
        sink.append(source.repeat_infinite());
        loop {
            if rx.try_recv().is_ok() {
                sink.stop();
                break;
            }
            if sink.empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });
}

pub fn stop_alarm() {
    if let Ok(mut guard) = STOP_SENDER.lock() {
        if let Some(tx) = guard.take() {
            let _ = tx.send(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_decodable_sound_file;
    use std::path::Path;

    #[test]
    fn accepts_a_rodio_decodable_sound_file() {
        assert!(is_decodable_sound_file(Path::new("audio/alarm.wav")));
    }

    #[test]
    fn rejects_a_missing_sound_file() {
        assert!(!is_decodable_sound_file(Path::new("audio/missing.wav")));
    }
}
