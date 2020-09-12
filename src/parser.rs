#[derive(Debug, PartialEq)]
pub enum MessageTokens {
    Hashtag(String),
    Text(String),
}

pub fn parse_message(text: &str) -> Vec<MessageTokens> {
    text.split_whitespace()
        .map(|word| match word.chars().next() {
            Some('#') => {
                let mut mutable_word = word.to_owned();
                mutable_word.remove(0);
                MessageTokens::Hashtag(mutable_word)
            }
            _ => MessageTokens::Text(word.to_owned()),
        })
        .collect()
}

#[cfg(test)]
mod parser_test {
    use super::*;

    #[test]
    fn single_hashtag() {
        let result = parse_message("#text");
        assert_eq!(result, vec!(MessageTokens::Hashtag("text".to_owned())));
    }

    #[test]
    fn single_text() {
        let result = parse_message("text");
        assert_eq!(result, vec!(MessageTokens::Text("text".to_owned())));
    }

    #[test]
    fn two_hashtags() {
        let result = parse_message("#first #second");
        assert_eq!(
            result,
            vec!(
                MessageTokens::Hashtag("first".to_owned()),
                MessageTokens::Hashtag("second".to_owned())
            )
        );
    }

    #[test]
    fn two_words() {
        let result = parse_message("one two");
        assert_eq!(
            result,
            vec!(
                MessageTokens::Text("one".to_owned()),
                MessageTokens::Text("two".to_owned())
            )
        );
    }

    #[test]
    fn hashtag_and_word() {
        let result = parse_message("#hash word");
        assert_eq!(
            result,
            vec!(
                MessageTokens::Hashtag("hash".to_owned()),
                MessageTokens::Text("word".to_owned())
            )
        )
    }
}
