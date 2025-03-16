use std::fs::File;
use std::io::Write;
use std::path::Path;
use clap::builder::Str;
use crate::park_miller_prng::ParkMiller;
use crate::st::st;
use crate::utils::{init_cli, process_files, ProcessResult, generate_wav, print_debug_information, print_amplitudes, plot_wav_amplitudes, count_bits_per_char, write_key_to_file, save_amplitudes_to_wav, WavFile, compare_amplitudes, read_key_from_file};
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
                print_debug_information(format!("{:#?}", data));
                let bits_per_char = count_bits_per_char(&*data.message)?;

                let samples_per_msg_bit: usize = (data.container.samples_num as f64 / (bits_per_char * data.message.len()) as f64).floor() as usize;
                print_debug_information(format!("N: {}", samples_per_msg_bit));

                if samples_per_msg_bit == 0 {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Недостаточно отсчетов, чтобы спрятать сообщение",
                    )));
                }


                let mut generator = ParkMiller::new();
                let psp = generator.generate_prs(samples_per_msg_bit);
                write_key_to_file(&psp, "key.csv")?;
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
                // print_debug_information(format!("{:#?}", new_wav));
                plot_wav_amplitudes(&new_wav, "stegacontainer.png")?;
                save_amplitudes_to_wav(&new_wav)?;
                compare_amplitudes(&data.container.amplitudes, &new_wav.amplitudes, 0.0001);
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
