use std::fs::File;
use std::io::Write;
use std::path::Path;
use crate::park_miller_prng::ParkMiller;
use crate::st::st;
use crate::utils::{init_cli, process_files, 
    ProcessResult, generate_wav, plot_wav_amplitudes, 
    count_bits_per_char, write_key_to_file, 
    save_amplitudes_to_wav, WavFile};
use crate::dest::dest;


mod utils;
mod park_miller_prng;
mod st;
mod dest;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = init_cli().unwrap_or_else(|e| e.exit());

    if matches.get_flag("generate-wav") {
        generate_wav(&matches)?;
        println!("WAV-файл был сгенерирован");
    } else {
        let data = process_files(&matches)?;

        match data {
            ProcessResult::Encrypt(data) => {
                let bits_per_char = count_bits_per_char(&*data.message)?;

                let samples_per_msg_bit: usize = (data.container.samples_num as f64 / (bits_per_char * data.message.len()) as f64).floor() as usize;

                if samples_per_msg_bit == 0 {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Недостаточно отсчетов, чтобы спрятать сообщение",
                    )));
                }


                let mut generator = ParkMiller::new();
                let psp = generator.generate_prs(samples_per_msg_bit);
                let key_filename = matches.get_one::<String>("key").unwrap().clone();
                write_key_to_file(&psp, key_filename.clone().as_str())?;

                println!("___ДАННЫЕ ДЛЯ ДЕКОДИРОВАНИЯ___");
                println!("n: {}\nm: {}\nN: {}", bits_per_char, data.message.len(), samples_per_msg_bit);
                println!("Ключ для декодирования был сохранен в {}", key_filename);
                println!("График исходного сигнала был сохранен в container.png");
                
                plot_wav_amplitudes(&data.container, "container.png")?;
                let result_amplitudes = st(&data, samples_per_msg_bit, bits_per_char, data.message.len(), psp);
                let new_wav = WavFile {
                    name: matches.get_one::<String>("stegacontainer").unwrap().clone(),
                    amplitudes: result_amplitudes,
                    bits_per_sample: data.container.bits_per_sample,
                    channels: data.container.channels,
                    sample_rate: data.container.sample_rate,
                    samples_num: data.container.samples_num,
                };

                println!("График измененного сигнала сохранен в stegacontainer.png");
                plot_wav_amplitudes(&new_wav, "stegacontainer.png")?;
                save_amplitudes_to_wav(&new_wav)?;
            }
            ProcessResult::Decrypt(data) => {
                let bits_per_char = *matches.get_one::<usize>("bits-per-char").unwrap();
                let message_len = *matches.get_one::<usize>("message-len").unwrap();
                let samples_per_msg_bit: usize = (data.container.samples_num as f64 / (bits_per_char * message_len) as f64).floor() as usize;
                let recovered_message = dest(&data, samples_per_msg_bit, bits_per_char, message_len);
                let mut message_file = File::create(Path::new(matches.get_one::<String>("message").unwrap()))?;
                message_file.write_all(&recovered_message)?;
                println!("Сообщение получено и сохранено в {}", matches.get_one::<String>("message").unwrap());
            }
        }
    }

    Ok(())
}
