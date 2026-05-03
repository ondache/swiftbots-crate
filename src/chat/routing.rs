use std::cmp::Ordering;
use regex::{Regex, RegexBuilder, escape, Error};
use crate::chat::types::{ChatCommand, MessageHandlerFunction};
use std::collections::{BinaryHeap, HashMap};
use http::Request;

pub enum TokenTrieValue <TBody> {
    Leaf(CompiledCommand<TBody>),
    Branch(TokenTrie<TBody>)
}

pub type TokenTrie <TBody> = HashMap<String, TokenTrieValue<TBody>>;

pub struct CompiledCommand <TBody> {
    pub re_command: Regex,
    pub command_name: String,
    pub handler_entry: MessageHandlerFunction<Request<TBody>>,
    pub command_len: usize
}

impl<TBody> Eq for CompiledCommand <TBody> {}

impl<TBody> PartialEq<Self> for CompiledCommand <TBody> {
    fn eq(&self, other: &Self) -> bool {
        self.command_len == other.command_len
    }
}

impl<TBody> PartialOrd<Self> for CompiledCommand <TBody> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<TBody> Ord for CompiledCommand <TBody> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.command_len.cmp(&other.command_len)
    }
}

pub const FINAL_INDICATOR: &str = "";

pub fn build_token_trie <TBody>(
    handlers: Vec<ChatCommand<Request<TBody>>>
) -> Result<TokenTrie<TBody>, Error> {
    let mut compiled_commands: Vec<CompiledCommand<TBody>> = vec![];
    for handler in handlers {
        for command in handler.commands {
            compiled_commands.push(CompiledCommand {
                re_command: compile_command_as_regex(command.as_str())?,
                command_len: command.len(),
                command_name: command,
                handler_entry: handler.callback.clone(),
            });
        }
    }
    let mut trie = HashMap::new();
    for command in compiled_commands {
        insert_token_trie(&mut trie, command);
    }
    Ok(trie)
}

pub fn insert_token_trie<TBody>(
    trie: &mut TokenTrie<TBody>, command: CompiledCommand<TBody>
) {
    let mut trie = trie;
    for word in command.command_name.to_lowercase().split_whitespace() {
        let entry = trie
            .entry(word.to_string())
            .or_insert(TokenTrieValue::Branch(TokenTrie::new()));
        if let TokenTrieValue::Branch(branch) = entry {
            trie = branch;
        } else {
            panic!("Expected branch, got leaf");
        }
    }
    trie.insert(FINAL_INDICATOR.to_string(), TokenTrieValue::Leaf(command));
}

pub fn search_token_trie<'a, TBody>(trie: &'a TokenTrie<TBody>, text: &'a str) -> Vec<&'a CompiledCommand<TBody>> {
    let mut current: Option<&TokenTrie<TBody>> = Some(trie);
    let mut matches: Vec<&'a CompiledCommand<TBody>> = vec![];
    let binding = text.to_lowercase();
    let mut splitted = binding.split_whitespace();
    while current.is_some() {
        if let Some(current) = current {
            if let Some(TokenTrieValue::Leaf(command)) = current.get(FINAL_INDICATOR) {
                matches.push(command);
            }
        }
        match splitted.next() {
            Some(word) => {
                current = match current {
                    Some(c) => match c.get(word) {
                        Some(TokenTrieValue::Branch(branch)) => Some(branch),
                        _ => None,
                    },
                    None => None,
                }
            }
            None => {
                break;
            }
        }
    }
    matches
}

pub fn search_matched_commands<'a, TBody> (trie: &'a TokenTrie<TBody>, text: &'a str) -> BinaryHeap<&'a CompiledCommand<TBody>> {
    let matches = search_token_trie(trie, text);
    let matches_queue: BinaryHeap<&'a CompiledCommand<TBody>> = BinaryHeap::from(matches);
    matches_queue
}

pub fn compile_command_as_regex (name: &str) -> Result<Regex, Error> {
    let mut name = name.to_string();
    if name.is_empty() {
        return Ok(RegexBuilder::new(r"^(.*)$")
            .dot_matches_new_line(true)
            .build()?);
    }
    name = name.split_whitespace().collect::<Vec<&str>>().join(" ");
    let escaped_name = escape(name.as_str());
    let pattern = format!(r"^{}(?:\s+(.*))?$", escaped_name);
    Ok(RegexBuilder::new(pattern.as_str())
        .dot_matches_new_line(true)
        .case_insensitive(true)
        .build()?)
}