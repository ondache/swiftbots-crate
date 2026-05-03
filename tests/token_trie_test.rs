use std::sync::Arc;
use swiftbots::chat::routing::{TokenTrie, CompiledCommand, insert_token_trie, search_matched_commands, compile_command_as_regex};
use swiftbots::chat::types::MessageHandlerFunction;
use http::Request;

fn mock_handler() -> MessageHandlerFunction<Request<()>> {
    Arc::new(|_, _| {
        Box::pin(async move {
            ()
        })
    })
}

fn try_on(trie: &TokenTrie<()>, text: &str) -> Option<String> {
    let mut matches = search_matched_commands(trie, text);
    matches.pop().map(|c| c.command_name.clone())
}

#[test]
fn routing_test() {
    let mut trie = TokenTrie::new();

    fn insert(trie: &mut TokenTrie<()>, command_name: &str) {
        let cmd = CompiledCommand {
            re_command: compile_command_as_regex(command_name).unwrap(),
            command_name: command_name.to_string(),
            handler_entry: mock_handler(),
            command_len: command_name.len(),
        };
        insert_token_trie(trie, cmd);
    }

    insert(&mut trie, "apple");
    insert(&mut trie, "cranberry");
    insert(&mut trie, "apple cranberry");
    insert(&mut trie, "苹果");
    insert(&mut trie, "apple cranberry cherry pineapple tuna");

    assert_eq!(try_on(&trie, "apple"), Some("apple".to_string()));
    assert_eq!(try_on(&trie, "cranberry"), Some("cranberry".to_string()));
    assert_eq!(try_on(&trie, "apple cranberry"), Some("apple cranberry".to_string()));
    assert_eq!(try_on(&trie, "apple pear"), Some("apple".to_string()));
    assert_eq!(try_on(&trie, "applecherry"), None);
    assert_eq!(try_on(&trie, "apple cherry"), Some("apple".to_string()));
    assert_eq!(try_on(&trie, "apple cranberrycherry"), Some("apple".to_string()));
    assert_eq!(try_on(&trie, "a"), None);
    assert_eq!(try_on(&trie, "cherry"), None);
    assert_eq!(try_on(&trie, "cherry apple"), None);
    assert_eq!(try_on(&trie, "pple"), None);
    assert_eq!(try_on(&trie, "苹果"), Some("苹果".to_string()));

    assert_eq!(try_on(&trie, "apple cranberry cherry pineapple tuna"), Some("apple cranberry cherry pineapple tuna".to_string()));
    assert_eq!(try_on(&trie, "apple cranberry cherry pineapple tuna salmon"), Some("apple cranberry cherry pineapple tuna".to_string()));
    assert_eq!(try_on(&trie, "apple cranberry cherry pineapple tunasalmon"), Some("apple cranberry".to_string()));

    insert(&mut trie, "");
    assert_eq!(try_on(&trie, "pple"), Some("".to_string()));
    assert_eq!(try_on(&trie, "apple cranberrycherry"), Some("apple".to_string()));
    assert_eq!(try_on(&trie, "苹果"), Some("苹果".to_string()));

    insert(&mut trie, "a b c d e f");
    // "a b c d e f" has 6 tokens.
    // "苹果" has 1 token but 6 bytes (in UTF-8).
    // If we used bytes, "苹果" might tie with "a b c d e f" (depending on implementation).
    // With tokens, "a b c d e f" is clearly longer.
    assert_eq!(try_on(&trie, "a b c d e f g"), Some("a b c d e f".to_string()));
}