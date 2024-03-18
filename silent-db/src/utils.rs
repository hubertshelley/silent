/// 字符串转换为驼峰命名
#[allow(dead_code)]
pub fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut next_upper = true;
    for c in s.chars() {
        if c == '_' {
            next_upper = true;
        } else if c.is_ascii_uppercase() {
            result.push(c);
            next_upper = false;
        } else if next_upper {
            result.push(c.to_ascii_uppercase());
            next_upper = false;
        } else {
            result.push(c.to_ascii_lowercase());
        }
    }
    result
}

/// 字符串转换为蛇形命名
#[allow(dead_code)]
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        if c.is_ascii_uppercase() {
            if !result.is_empty() {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("hello_world"), "HelloWorld");
        assert_eq!(to_camel_case("rust_programming"), "RustProgramming");
        assert_eq!(to_camel_case("github_copilot"), "GithubCopilot");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(to_snake_case("RustProgramming"), "rust_programming");
        assert_eq!(to_snake_case("GithubCopilot"), "github_copilot");
    }
}
