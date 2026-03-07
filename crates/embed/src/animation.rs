use std::time::Duration;

pub fn typewriter_delay_ms(char_count: usize, speed_ms_per_char: u32) -> Duration {
    Duration::from_millis((char_count as u64) * (speed_ms_per_char as u64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delay_scales_with_char_count() {
        assert_eq!(typewriter_delay_ms(10, 5), Duration::from_millis(50));
    }
}
