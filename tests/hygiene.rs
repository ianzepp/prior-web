#![allow(clippy::absurd_extreme_comparisons)]

use std::fs;
use std::path::Path;

const MAX_UNWRAP: usize = 0;
const MAX_EXPECT: usize = 1;
const MAX_PANIC: usize = 0;
const MAX_UNREACHABLE: usize = 0;
const MAX_TODO: usize = 0;
const MAX_UNIMPLEMENTED: usize = 0;

struct SourceFile {
    content: String,
}

fn source_files() -> Vec<SourceFile> {
    let mut files = Vec::new();
    collect_rs_files(Path::new("src"), &mut files);
    files
}

fn collect_rs_files(dir: &Path, out: &mut Vec<SourceFile>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name.ends_with("_test.rs") || name.ends_with(".test.rs") {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&path) {
                out.push(SourceFile { content });
            }
        }
    }
}

fn count_in_source(files: &[SourceFile], pattern: &str) -> usize {
    files
        .iter()
        .map(|file| {
            file.content
                .lines()
                .filter(|line| line.contains(pattern))
                .count()
        })
        .sum()
}

fn assert_budget(name: &str, count: usize, max: usize) {
    assert!(
        count <= max,
        "{name} budget exceeded: found {count}, max {max}."
    );
}

#[test]
fn unwrap_budget() {
    let count = count_in_source(&source_files(), ".unwrap()");
    assert_budget(".unwrap()", count, MAX_UNWRAP);
}

#[test]
fn expect_budget() {
    let count = count_in_source(&source_files(), ".expect(");
    assert_budget(".expect(", count, MAX_EXPECT);
}

#[test]
fn panic_budget() {
    let count = count_in_source(&source_files(), "panic!(");
    assert_budget("panic!(", count, MAX_PANIC);
}

#[test]
fn unreachable_budget() {
    let count = count_in_source(&source_files(), "unreachable!(");
    assert_budget("unreachable!(", count, MAX_UNREACHABLE);
}

#[test]
fn todo_budget() {
    let count = count_in_source(&source_files(), "todo!(");
    assert_budget("todo!(", count, MAX_TODO);
}

#[test]
fn unimplemented_budget() {
    let count = count_in_source(&source_files(), "unimplemented!(");
    assert_budget("unimplemented!(", count, MAX_UNIMPLEMENTED);
}
