use rand::{distributions::Alphanumeric, thread_rng, Rng as _};

/*
https://github.com/curl/curl/blob/curl-7_79_1/lib/mime.c#L1320
https://github.com/curl/curl/blob/curl-7_79_1/lib/rand.c#L150
https://github.com/curl/curl/blob/curl-7_79_1/lib/mime.h#L27
*/
pub fn generate() -> String {
    format!(
        "------------------------{}",
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect::<String>()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        assert_eq!(generate().len(), 24 + 16);
    }
}
