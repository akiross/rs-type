use std::path::Path;

static DICTIONARY: &[&str] = &[
    "jj", "ff", "jj", "ff", "jjj", "fff", "jfj", "fjj", "ffj", "jff", "ciao", "belli", "come",
    "state",
];

pub enum Enemies {
    Some(String),
    LevelComplete,
    GameComplete,
}

pub trait WordProducer {
    /// Produces the n-th word of the l-th level
    fn next_word(&mut self, l: usize, n: usize) -> Enemies;
}

pub struct CodeDict;

impl WordProducer for CodeDict {
    fn next_word(&mut self, l: usize, n: usize) -> Enemies {
        if l > 2 {
            Enemies::GameComplete
        } else if n < 10 {
            // Generate random word of given size
            Enemies::Some(DICTIONARY[n % DICTIONARY.len()].to_owned())
        } else {
            Enemies::LevelComplete
        }
    }
}

pub struct KTouchParser {
    lessons: Vec<Vec<String>>,
}

impl KTouchParser {
    pub fn new(src: &Path) -> Self {
        let contents = std::fs::read_to_string(src).expect("Cannot open file");

        let doc = roxmltree::Document::parse(&contents).unwrap();
        let lessons = doc
            .root()
            .first_element_child()
            .expect("Unable to find course root")
            .children()
            .find(|n| n.tag_name().name() == "lessons")
            .expect("Unable to find lessons tag");

        let lessons = lessons
            .children()
            .filter(roxmltree::Node::is_element)
            .take(3)
            .map(|lesson| {
                lesson
                    .children()
                    .filter(roxmltree::Node::is_element)
                    .find(|c| c.tag_name().name() == "text")
                    .expect("Unable to find lesson's text")
                    .text()
                    .expect("There is no text in this lesson!")
                    .split_whitespace()
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        KTouchParser { lessons }
    }
}

impl WordProducer for KTouchParser {
    fn next_word(&mut self, l: usize, n: usize) -> Enemies {
        if l < self.lessons.len() {
            if n < self.lessons[l].len() {
                Enemies::Some(self.lessons[l][n].clone())
            } else {
                Enemies::LevelComplete
            }
        } else {
            Enemies::GameComplete
        }
    }
}
