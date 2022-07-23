use crate::markdown_service::markdown_service::{MarkdownService, PostMetadata};
use crate::posts::{get_posts, PostFile};
use chrono::NaiveDateTime;
use once_cell::sync::Lazy;
use regex::Regex;
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Post {
    pub metadata: PostMetadata,
    pub raw_content: &'static str,
    pub desc: String,
    pub modified_time: String,
    pub filename: &'static str,
}

pub struct PostsContainer {
    posts_map: HashMap<String, Vec<Post>>,
}

#[derive(Clone)]
pub enum FilterTag {
    All,
    Tag(String),
}

const MAX_DESC_LENGTH: usize = 600;

pub fn find_char_boundary(s: &str, index: usize) -> usize {
    if s.len() <= index {
        return index;
    }

    let mut new_index = index;
    while !s.is_char_boundary(new_index) {
        new_index += 1;
    }

    new_index
}

impl PostsContainer {
    pub fn new() -> PostsContainer {
        let mut posts_map = HashMap::from([]);

        for (key, value) in get_posts() {
            posts_map.insert(key, PostsContainer::read_posts_into_memo(value));
        }

        PostsContainer { posts_map }
    }

    pub fn get_posts_by_key(&self, key: String) -> Vec<Post> {
        self.posts_map.get(&key).unwrap().clone()
    }

    pub fn get_posts(&self) -> Vec<Post> {
        let mut target_posts: Vec<Post> = vec![];

        for posts in self.posts_map.values() {
            target_posts.extend((*posts).clone())
        }

        target_posts
    }

    pub fn trim_useless_symbol(content: &'static str) -> String {
        Regex::new(r#"([\n]|```[^`]+```|`[^`]+`)"#)
            .unwrap()
            .replace_all(content, "")
            .into_owned()
    }

    fn read_posts_into_memo(posts: Vec<PostFile>) -> Vec<Post> {
        let mut posts = posts
            .clone()
            .into_iter()
            .map(|post| {
                let markdown_service = MarkdownService::new(post.content.to_string());
                let metadata = markdown_service.extract_metadata().expect(
                    "Please make sure the post has metadata which declare using block syntax.",
                );
                let parsed_content = PostsContainer::trim_useless_symbol(post.content);
                let parsed_content_length = parsed_content.len();
                let slice_desc_length = if parsed_content_length > MAX_DESC_LENGTH {
                    MAX_DESC_LENGTH
                } else {
                    parsed_content_length
                };
                let desc = parsed_content[..find_char_boundary(&parsed_content, slice_desc_length)]
                    .to_string();
                let modified_secs = (post.modified_time / 1000) as i64;
                let modified_time = NaiveDateTime::from_timestamp(modified_secs, 0);
                let modified_time = modified_time.format("%a, %b %e %Y").to_string();

                Post {
                    metadata,
                    raw_content: post.content,
                    desc,
                    modified_time,
                    filename: post.filename,
                }
            })
            .collect::<Vec<Post>>();

        posts.sort_by(|a, b| {
            if a.metadata.pined {
                Ordering::Less
            } else if b.metadata.pined {
                Ordering::Greater
            } else {
                a.modified_time.cmp(&b.modified_time)
            }
        });

        posts
    }
}

pub static POST_CONTAINER: Lazy<PostsContainer> = Lazy::new(|| PostsContainer::new());
