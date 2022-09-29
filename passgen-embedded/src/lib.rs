#![cfg_attr(not(test), no_std)]
#![feature(lang_items)]


extern crate rand;

use core::panic::PanicInfo;
use core::slice::from_raw_parts_mut;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rand::RngCore;
use crate::symbols::POOL;

mod symbols;


#[cfg(not(test))]
#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[cfg(not(test))]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}


fn get_random_value(rng: &mut SmallRng, low: usize, high: usize) -> usize {
    ((rng.next_u32() as f32 / u32::MAX as f32) * (high - low) as f32 + low as f32) as usize
}


/// Get a symbol from all symbols in the pool
fn get_symbol_by_offset(random_index: usize) -> u8 {
    let mut index = random_index;
    let slices = POOL;
    let mut s = slices[0];

    for slice in slices {
        let len = slice.len();

        if index < len {
            s = slice;
            break;
        } else {
            index -= len;
        }
    }
    s[index]
}

fn shuffle(length: usize, buffer: &mut [u8], rng: &mut SmallRng) -> () {
    if length <= 1 {
        return;
    }

    for i in 0..length {
        let j = get_random_value(rng, 0, length);
        let symbol = buffer[j];
        buffer[j] = buffer[i];
        buffer[i] = symbol;
    }
}

/// Generate a random password.
fn generate(buffer: &mut [u8], length: usize, random_state: u64) {
    let mut small_rng = SmallRng::seed_from_u64(random_state);
    let total_dictionary_len: usize = POOL.iter().map(|slice| slice.len()).sum();

    // Generate placeholders with category index or MAX value as random element
    // We need each category to participate once and all others to be random
    for i in 0..POOL.len() {
        buffer[i] = i as u8;
    }
    for i in POOL.len()..length as usize {
        buffer[i] = u8::MAX;
    }
    shuffle(length as usize, buffer, &mut small_rng);

    for index in 0..length as usize {
        if buffer[index] == u8::MAX {
            let random_index = get_random_value(&mut small_rng, 0, total_dictionary_len);
            buffer[index] = get_symbol_by_offset(random_index as usize)
        } else {
            let category = POOL[buffer[index] as usize];
            let random_index = get_random_value(&mut small_rng, 0, category.len());
            buffer[index] = category[random_index];
        }
    }
}

#[repr(C)]
pub struct GenerationResult {
    success: bool,
    error_string: *const u8,
}

#[no_mangle]
pub unsafe extern "C" fn generate_password(buffer: *mut u8, length: usize, random_state: u64) -> GenerationResult {
    if length == 0 {
        return GenerationResult {
            success: false,
            error_string: b"len(password) == 0.\0".as_ptr(),
        };
    }

    return match buffer.as_mut() {
        None => {
            GenerationResult {
                success: false,
                error_string: b"Buffer is nullptr.\0".as_ptr(),
            }
        }
        Some(buffer) => {
            let parsed_buffer = from_raw_parts_mut(buffer, length);

            generate(parsed_buffer, length, random_state);
            GenerationResult {
                success: true,
                error_string: b"\0".as_ptr(),
            }
        }
    };
}


#[cfg(test)]
mod tests {
    use rand::rngs::SmallRng;
    use crate::{generate, get_random_value, get_symbol_by_offset, SeedableRng, shuffle};

    #[test]
    fn test_random_numbers() {
        let mut rng = SmallRng::seed_from_u64(1);
        let result = get_random_value(&mut rng, 0, 100);
        assert_eq!(result < 101, true);
        assert_eq!(result > 0, true);
    }

    #[test]
    fn test_get_symbol_by_offset() {
        let indices: [usize; 4] = [7, 45, 22, 17];
        let result = indices.map(get_symbol_by_offset);
        assert_eq!(result, [57, 81, 114, 107]);
    }

    #[test]
    fn test_shuffle() {
        let mut rng = SmallRng::seed_from_u64(42);
        let mut data = [2, 12, 85, 0, 6];
        shuffle(data.len(), &mut data, &mut rng);
        assert_eq!(data, [2, 85, 6, 0, 12]);
    }

    #[test]
    fn test_generate() {
        let mut buffer = [0; 12];
        generate(&mut buffer, 12, 42);
        assert_eq!(buffer, [75, 62, 40, 78, 42, 55, 122, 38, 103, 55, 64, 65]);
    }
}