use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;
use clap::{Arg, ArgAction, ArgMatches, Command, Error, ArgGroup};
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use plotters::prelude::*;

pub fn init_cli() -> Result<ArgMatches, Error> {
    let ret = Command::new("Steganography third lab")
        .arg(
            Arg::new("encrypt")
                .help("Кодирование сообщения")
                .long("encrypt")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("decrypt")
                .help("Декодирование сообщения")
                .long("decrypt")
                .action(ArgAction::SetTrue),
        )
        .group(
            ArgGroup::new("mode")
                .args(&["encrypt", "decrypt", "generate-wav"])
                .required(true)
                .multiple(false),
        )
        .arg(
            Arg::new("generate-wav")
                .help("Генерирование WAV-файла с указанными параметрами")
                .long("generate-wav")
                .short('g')
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("duration")
                .help("Длина генерируемого WAV-файла")
                .long("duration")
                .short('d')
                .action(ArgAction::Set)
                .value_parser(clap::value_parser!(f32))
        )
        .arg(
            Arg::new("channels")
                .help("Количество каналов генерируемого WAV-файла (1 - моно, 2 - стерео)")
                .long("channels")
                .action(ArgAction::Set)
                .value_parser(clap::value_parser!(u16)),
        )
        .arg(
            Arg::new("sample-rate")
                .help("Частота дискретизации генерируемого WAV-файла")
                .long("sample-rate")
                .short('r')
                .action(ArgAction::Set)
                .default_value("44100")
                .value_parser(clap::value_parser!(u32))
        )
        .arg(
            Arg::new("name")
                .help("Имя генерируемого WAV-файла")
                .long("name")
                .short('n')
                .action(ArgAction::Set)
        )
        .group(
            ArgGroup::new("wav-generation")
                .args(&["generate-wav", "duration", "channels", "name", "channels", "sample-rate"])
                .multiple(true)
                .requires_all(&["duration", "channels", "name", "sample-rate"]),
        )
        .arg(
            Arg::new("container")
                .help("Путь до контейнера WAV-формата")
                .long("container")
                .short('c')
                .default_value("container.wav")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("stegacontainer")
                .help("Путь до стегаконтейнера в WAV-формате")
                .long("stegacontainer")
                .short('s')
                .default_value("stegacontainer.wav")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("message")
                .help("Путь до файла с сообщением")
                .long("message")
                .short('m')
                .default_value("message.txt")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("key")
                .long("key")
                .short('k')
                .action(ArgAction::Set)
                .default_value("key.csv")
                .requires("decrypt")
        )
        .arg(
            Arg::new("bits-per-char")
                .help("Количество бит на символ вытаскиваемого сообщения")
                .long("bits-per-char")
                .short('b')
                .action(ArgAction::Set)
                .value_parser(clap::value_parser!(usize))
                .requires("decrypt")
        )
        .arg(
            Arg::new("message-len")
                .help("Длина вытаскиваемого сообщения")
                .long("message-len")
                .short('l')
                .action(ArgAction::Set)
                .value_parser(clap::value_parser!(usize))
                .requires("decrypt")
        )
        .try_get_matches();
    ret
}

fn read_file(file: Result<File, io::Error>, buffer: &mut Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = file.map_err(|e| {
        eprintln!("Ошибка при чтении файла: {}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?;

    let file_size = file.metadata()?.len();
    buffer.resize(file_size as usize, 0);

    file.read_exact(buffer).map_err(|e| {
        eprintln!("Ошибка при чтении файла: {}",  e);
        Box::new(io::Error::new(io::ErrorKind::InvalidInput, e)) as Box<dyn std::error::Error>
    })?;

    Ok(())
}


// Учитывается, что у нас в сообщении не смешиваются латиница и кириллица. (НЕ ФАКТ ЧТО РАБОТАЕТ)
pub fn count_bits_per_char(bytes: &[u8]) -> Result<usize, Box<dyn std::error::Error>> {
    let mut max_bits_per_char: usize = 0;
    match std::str::from_utf8(bytes) {
        Ok(s) => {
            for c in s.chars() {
                let bytes = c.len_utf8();
                let bits = bytes * 8;
                if bits > max_bits_per_char {
                    max_bits_per_char = bits;
                }
                // print_debug_information(format!("Символ: {}, Байты: {}, Биты: {}", c, bytes, bits));
            }
        }
        Err(_) => {
            return Err(Box::from("Ошибка: массив байтов содержит невалидный UTF-8"))
        }
    }
    Ok(max_bits_per_char)
}
pub fn process_files(matches: &ArgMatches) -> Result<ProcessResult, Box<dyn std::error::Error>> {
    if matches.get_flag("encrypt") {
        let wav_path = matches.get_one::<String>("container").unwrap();
        let message_path = matches.get_one::<String>("message").unwrap();
        let message_file = File::open(Path::new(message_path));

        let container = get_wav_file_data(wav_path)?;
        let mut message: Vec<u8> = Vec::new();
        read_file(message_file, &mut message)?;

        Ok(ProcessResult::Encrypt (EncryptData {
            container,
            message,
        }))
    } else {
        let container_wav_path = matches.get_one::<String>("container").unwrap();
        let stegocontainer_wav_path = matches.get_one::<String>("stegacontainer").unwrap();
        let key_path = matches.get_one::<String>("key").unwrap();
        let container = get_wav_file_data(container_wav_path)?;
        let stegocontainer = get_wav_file_data(stegocontainer_wav_path)?;
        let key = read_key_from_file(key_path)?;

        Ok(ProcessResult::Decrypt (DecryptData {
            container,
            stegocontainer,
            key
        }))
    }
}

fn get_wav_file_data(wav_path: &String) -> Result<WavFile, Box<dyn std::error::Error>> {
    let mut wav = WavReader::open(Path::new(wav_path))?;

    let spec = wav.spec();

    let samples: Vec<i16> = wav.samples::<i16>()
        .map(|result| result.unwrap())
        .collect();

    let max_amplitude = i16::MAX as f64;
    let amplitudes: Vec<f64> = samples
        .into_iter()
        .map(|sample| sample as f64 / max_amplitude)
        .collect();

    let data = WavFile {
        name: wav_path.clone(),
        amplitudes,
        bits_per_sample: spec.bits_per_sample,
        channels: spec.channels,
        sample_rate: spec.sample_rate,
        samples_num: wav.len(),
    };
    Ok(data)
}

pub fn save_amplitudes_to_wav(new_wav: &WavFile) -> Result<(), Box<dyn std::error::Error>> {
    let spec = WavSpec {
        channels: new_wav.channels,
        sample_rate: new_wav.sample_rate,
        bits_per_sample: new_wav.bits_per_sample,
        sample_format: SampleFormat::Int,
    };

    let mut writer = WavWriter::create(Path::new(&new_wav.name), spec)?;

    for amplitude in &new_wav.amplitudes {
        // Ограничиваем амплитуды диапазоном [-1.0, 1.0]
        let amplitude = amplitude.clamp(-1.0, 1.0);
        let sample = (amplitude * i16::MAX as f64) as i16;
        writer.write_sample(sample)?;
    }

    writer.finalize()?;
    Ok(())
}

pub fn plot_wav_amplitudes(wav: &WavFile, plotname: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(plotname, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption(format!("Амплитуды файла {}", wav.name), ("sans-serif", 24))
        .build_cartesian_2d(0..wav.amplitudes.len(), -1.2f64..1.2f64)?;
    chart.configure_mesh().draw().unwrap();
    let step = 100;
    let sampled_amplitudes: Vec<_> = wav.amplitudes.iter().step_by(step).cloned().collect();
    chart
        .draw_series(LineSeries::new(
            sampled_amplitudes.iter().enumerate().map(|(i, &v)| (i * step, v)),
            &BLACK
        )).unwrap();
    Ok(())
}

pub fn generate_wav(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let spec = WavSpec {
        channels: *matches.get_one::<u16>("channels").unwrap(),
        sample_rate: *matches.get_one::<u32>("sample-rate").unwrap(),
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let duration: f32 = *matches.get_one::<f32>("duration").unwrap();

    let mut writer = WavWriter::create(matches.get_one::<String>("name").unwrap(), spec)?;

    for t in 0..((spec.sample_rate as f32 * duration) as u32) {
        let time = t as f32 / spec.sample_rate as f32;
        let sample = (time * 440.0 * 2.0 * std::f32::consts::PI).sin();
        let normalized_sample = (sample * i16::MAX as f32) as i16;
        writer.write_sample(normalized_sample)?;
    }
    writer.finalize()?;
    Ok(())
}

pub fn write_key_to_file(key: &[i16], filename: &str) -> std::io::Result<()> {
    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);

    for (i, &value) in key.iter().enumerate() {
        write!(writer, "{}", value)?;
        if i != key.len() - 1 {
            write!(writer, ",")?;
        }
    }
    Ok(())
}

pub fn read_key_from_file(filename: &str) -> std::io::Result<Vec<i16>> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);

    let mut line = String::new();
    reader.read_line(&mut line)?;


    let key: Vec<i16> = line
        .trim()
        .split(',')
        .map(|s| s.parse::<i16>().unwrap())
        .collect();

    Ok(key)
}


pub enum ProcessResult {
    Encrypt(EncryptData),
    Decrypt(DecryptData),
}
#[derive(Debug)]
pub struct EncryptData {
    pub container: WavFile,
    pub message: Vec<u8>,
}

pub struct DecryptData {
    pub container: WavFile,
    pub stegocontainer: WavFile,
    pub key: Vec<i16>,
}

#[derive(Debug)]
pub struct WavFile {
    pub name: String,
    pub amplitudes: Vec<f64>,
    pub bits_per_sample: u16,
    pub channels: u16,
    pub sample_rate: u32,
    pub samples_num: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;

    #[test]
    fn test_write_key_to_file() {
        let key = vec![1, -1, 2, -2, 3];
        let filename = "test_key.txt";

        let result = write_key_to_file(&key, filename);
        assert!(result.is_ok(), "Функция вернула ошибку: {:?}", result);

        let mut file = File::open(filename).expect("Не удалось открыть файл");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Не удалось прочитать файл");

        let expected_contents = "1,-1,2,-2,3";
        assert_eq!(contents, expected_contents, "Содержимое файла не совпадает с ожидаемым");

        fs::remove_file(filename).expect("Не удалось удалить тестовый файл");
    }

    #[test]
    fn test_write_key_to_file_invalid_path() {
        let key = vec![1, -1, 2, -2, 3];
        let filename = "/invalid/path/test_key.txt"; // Недопустимый путь

        let result = write_key_to_file(&key, filename);
        assert!(result.is_err(), "Функция должна вернуть ошибку для недопустимого пути");
    }
}