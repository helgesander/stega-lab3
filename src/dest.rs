use bitvec::order::Msb0;
use bitvec::prelude::BitVec;
use crate::utils::DecryptData;

pub fn dest(data: &DecryptData, samples_per_msg_bit: usize, n: usize, m: usize) -> Vec<u8> {
    let mut recovered_message_bitvec: BitVec<_, Msb0> = BitVec::new();

    for i in 0..n * m {
        let start = i * samples_per_msg_bit;
        let end = (i + 1) * samples_per_msg_bit;

        let segment_stego = &data.stegocontainer.amplitudes[start..end];
        let segment_original = &data.container.amplitudes[start..end];

        let res: Vec<f64> = segment_stego.iter()
            .zip(segment_original.iter())
            .map(|(&x, &y)| (x - y) / (y + 2.0))
            .collect();

        let b = if (res[0] > 0.0 && data.key[0] > 0) || (res[0] < 0.0 && data.key[0] < 0) {
            true
        } else {
            false
        };
        recovered_message_bitvec.push(b);
    }

    recovered_message_bitvec.into_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitvec::prelude::*;
    use crate::utils::{DecryptData, WavFile};

    // Вспомогательная функция для создания тестовых данных
    fn create_test_data() -> DecryptData {
        let container = WavFile {
            name: "original.wav".to_string(),
            amplitudes: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8],
            bits_per_sample: 16,
            channels: 1,
            sample_rate: 44100,
            samples_num: 8,
        };

        let stegocontainer = WavFile {
            name: "stego.wav".to_string(),
            amplitudes: vec![0.1001, 0.1999, 0.3001, 0.3999, 0.5001, 0.5999, 0.7001, 0.7999],
            bits_per_sample: 16,
            channels: 1,
            sample_rate: 44100,
            samples_num: 8,
        };

        let key = vec![1, -1, 1, -1, 1, -1, 1, -1];  // Псевдослучайная последовательность (PSP)

        DecryptData {
            container,
            stegocontainer,
            key,
        }
    }

    #[test]
    fn test_dest_with_non_empty_message() {
        let data = create_test_data();
        let n = 8;  // 8 бит в сообщении
        let m = 1;  // 1 байт в сообщении
        let samples_per_msg_bit = 1;  // 1 сэмпл на бит

        // Вызов функции dest для извлечения сообщения
        let recovered_message = dest(&data, n, m, samples_per_msg_bit);

        // Ожидаемое сообщение (в битах)
        let expected_message = vec![0b10101010];  // Пример сообщения (1 байт)

        // Проверяем, что извлеченное сообщение совпадает с ожидаемым
        assert_eq!(recovered_message, expected_message);
    }

    #[test]
    fn test_dest_with_empty_message() {
        let container = WavFile {
            name: "original.wav".to_string(),
            amplitudes: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8],
            bits_per_sample: 16,
            channels: 1,
            sample_rate: 44100,
            samples_num: 8,
        };

        let stegocontainer = WavFile {
            name: "stego.wav".to_string(),
            amplitudes: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8],  // Амплитуды не изменены
            bits_per_sample: 16,
            channels: 1,
            sample_rate: 44100,
            samples_num: 8,
        };

        let key = vec![1, -1, 1, -1, 1, -1, 1, -1];  // Псевдослучайная последовательность (PSP)

        let data = DecryptData {
            container,
            stegocontainer,
            key,
        };

        let n = 8;  // 8 бит в сообщении
        let m = 0;  // 0 байт в сообщении
        let samples_per_msg_bit = 1;  // 1 сэмпл на бит

        // Вызов функции dest для извлечения сообщения
        let recovered_message = dest(&data, n, m, samples_per_msg_bit);

        // Ожидаемое сообщение (пустое)
        let expected_message: Vec<u8> = vec![];

        // Проверяем, что извлеченное сообщение пустое
        assert_eq!(recovered_message, expected_message);
    }
}