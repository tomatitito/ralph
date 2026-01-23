use crate::config::TokenEstimationMethod;

/// Token counter for estimating context size
pub struct TokenCounter {
    method: TokenEstimationMethod,
    bpe: Option<tiktoken_rs::CoreBPE>,
}

impl TokenCounter {
    /// Create a new TokenCounter with the specified estimation method
    pub fn new(method: TokenEstimationMethod) -> Self {
        let bpe = if method == TokenEstimationMethod::Tiktoken {
            tiktoken_rs::cl100k_base().ok()
        } else {
            None
        };

        Self { method, bpe }
    }

    /// Estimate the token count for the given text
    pub fn count(&self, text: &str) -> usize {
        match self.method {
            TokenEstimationMethod::Tiktoken => {
                if let Some(ref bpe) = self.bpe {
                    bpe.encode_with_special_tokens(text).len()
                } else {
                    // Fallback to byte ratio if tiktoken fails to initialize
                    text.len() / 4
                }
            }
            TokenEstimationMethod::ByteRatio => text.len() / 4,
            TokenEstimationMethod::CharRatio => text.chars().count() / 4,
        }
    }
}

impl Default for TokenCounter {
    fn default() -> Self {
        Self::new(TokenEstimationMethod::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_ratio_estimation() {
        let counter = TokenCounter::new(TokenEstimationMethod::ByteRatio);
        // 20 bytes / 4 = 5 tokens
        assert_eq!(counter.count("12345678901234567890"), 5);
    }

    #[test]
    fn test_char_ratio_estimation() {
        let counter = TokenCounter::new(TokenEstimationMethod::CharRatio);
        // 20 chars / 4 = 5 tokens
        assert_eq!(counter.count("12345678901234567890"), 5);
    }

    #[test]
    fn test_tiktoken_estimation() {
        let counter = TokenCounter::new(TokenEstimationMethod::Tiktoken);
        // "Hello, world!" should be a few tokens
        let count = counter.count("Hello, world!");
        assert!(count > 0);
        assert!(count < 10);
    }

    #[test]
    fn test_estimates_within_range() {
        let tiktoken = TokenCounter::new(TokenEstimationMethod::Tiktoken);
        let byte_ratio = TokenCounter::new(TokenEstimationMethod::ByteRatio);
        let char_ratio = TokenCounter::new(TokenEstimationMethod::CharRatio);

        let text = "This is a test of the token counting system with some longer text to ensure accurate estimates.";

        let tk_count = tiktoken.count(text);
        let byte_count = byte_ratio.count(text);
        let char_count = char_ratio.count(text);

        // All estimates should be positive
        assert!(tk_count > 0);
        assert!(byte_count > 0);
        assert!(char_count > 0);

        // Estimates should be in a reasonable range of each other (within 3x)
        assert!(tk_count < byte_count * 3 + 1);
        assert!(byte_count < tk_count * 3 + 1);
        assert!(char_count < tk_count * 3 + 1);
    }
}
