use bitvec::prelude::*;
use crate::utils::EncryptData;


pub fn st(data: &EncryptData, samples_per_msg_bit: usize, n: usize, m: usize, psp: Vec<i16>) -> Vec<f64> {
    let mut result_amplitudes = data.container.amplitudes.clone();

    let msg_bits: BitVec<_, Msb0> = BitVec::from_slice(&data.message);
    let mut msg_bits_iter = msg_bits.into_iter();

    for i in 0..(n * m) {
        let pspmes: Vec<f64> = if msg_bits_iter.next() == Some(false) {
            psp.iter().map(|&x| -x as f64 * 0.0005).collect()
        } else {
            psp.iter().map(|&x| x as f64 * 0.0005).collect()
        };

        let start = i * samples_per_msg_bit;
        let end = (i + 1) * samples_per_msg_bit;

        for j in start..end {
            let original_amp = result_amplitudes[j];
            // println!("{} -> {}", data.container.amplitudes[j], original_amp + pspmes[j - start] * (original_amp + 2.0));
            result_amplitudes[j] = original_amp + pspmes[j - start] * (original_amp + 2.0);
        }
    }

    result_amplitudes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{EncryptData, WavFile};
    use crate::park_miller_prng::ParkMiller;

    // Вспомогательная функция для создания тестовых данных
    fn create_test_data() -> (EncryptData, Vec<i16>) {
        let container = WavFile {
            name: "test.wav".to_string(),
            amplitudes: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8],
            bits_per_sample: 16,
            channels: 1,
            sample_rate: 44100,
            samples_num: 8,
        };

        let message = vec![0b10101010];  // Пример сообщения (1 байт)
        let encrypt_data = EncryptData {
            container,
            message,
        };

        // Генерация псевдослучайной последовательности (PSP)
        let mut generator = ParkMiller::new();
        let psp = generator.generate_prs(8);  // 8 сэмплов на бит

        (encrypt_data, psp)
    }

    #[test]
    fn test_st_with_non_empty_message() {
        let (encrypt_data, psp) = create_test_data();
        let bits_per_char = 8;  // 8 бит на символ (ASCII)
        let samples_per_msg_bit = encrypt_data.container.samples_num as usize / (bits_per_char * encrypt_data.message.len());

        // Вызов функции st для создания стего-контейнера
        let stego_amplitudes = st(&encrypt_data, samples_per_msg_bit as usize, bits_per_char, encrypt_data.message.len(), psp);

        // Проверяем, что длина массива амплитуд не изменилась
        assert_eq!(stego_amplitudes.len(), encrypt_data.container.amplitudes.len());

        // Проверяем, что амплитуды изменились
        assert_ne!(stego_amplitudes, encrypt_data.container.amplitudes);

        // Проверяем, что изменения амплитуд соответствуют ожидаемым
        for i in 0..stego_amplitudes.len() {
            let original_amp = encrypt_data.container.amplitudes[i];
            let stego_amp = stego_amplitudes[i];
            assert_ne!(original_amp, stego_amp);  // Амплитуды должны измениться
        }
    }

    #[test]
    fn test_st_with_empty_message() {
        let container = WavFile {
            name: "test.wav".to_string(),
            amplitudes: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8],
            bits_per_sample: 16,
            channels: 1,
            sample_rate: 44100,
            samples_num: 8,
        };

        let message = vec![];  // Пустое сообщение
        let encrypt_data = EncryptData {
            container,
            message,
        };

        // Генерация псевдослучайной последовательности (PSP)
        let mut generator = ParkMiller::new();
        let psp = generator.generate_prs(8);  // 8 сэмплов на бит

        let bits_per_char = 8;  // 8 бит на символ (ASCII)
        let samples_per_msg_bit = encrypt_data.container.samples_num as usize / (bits_per_char * encrypt_data.message.len().max(1));

        // Вызов функции st для создания стего-контейнера
        let stego_amplitudes = st(&encrypt_data, samples_per_msg_bit as usize, bits_per_char, encrypt_data.message.len(), psp);

        // Проверяем, что амплитуды не изменились, так как сообщение пустое
        assert_eq!(stego_amplitudes, encrypt_data.container.amplitudes);
    }
}