/// 配置文件只读查看器。
///
/// 以模态对话框展示当前生效的 Shell 配置文件内容，
/// 并提供轻量的语法高亮（注释、字符串、关键字、变量）。
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::i18n::t;
use crate::state::app_state::AppState;

/// 行首关键字集合（bash / zsh / fish 常见指令）。
const KEYWORDS: &[&str] = &[
    "alias", "export", "set", "unset", "source", "unalias", "function", "abbr", "bind",
    "complete", "eval",
];

/// 词法单元类型，对应 CSS 类名。
#[derive(Debug, Clone, Copy, PartialEq)]
enum Tok {
    Plain,
    Comment,
    Str,
    Keyword,
    Variable,
}

fn tok_class(tok: Tok) -> &'static str {
    match tok {
        Tok::Plain => "",
        Tok::Comment => "tok-comment",
        Tok::Str => "tok-string",
        Tok::Keyword => "tok-keyword",
        Tok::Variable => "tok-variable",
    }
}

/// 将一行配置切分为（类型，文本）序列。
///
/// 展示用的简化词法分析：整行注释、引号字符串、行首关键字、`$变量`，
/// 其余按普通文本处理；不追求与 shell 语法完全一致。
fn tokenize_line(line: &str) -> Vec<(Tok, String)> {
    if line.trim_start().starts_with('#') {
        return vec![(Tok::Comment, line.to_string())];
    }

    let chars: Vec<char> = line.chars().collect();
    let mut out: Vec<(Tok, String)> = Vec::new();
    let mut plain = String::new();
    let mut at_line_start = true;
    let mut i = 0;

    let flush = |out: &mut Vec<(Tok, String)>, plain: &mut String| {
        if !plain.is_empty() {
            out.push((Tok::Plain, std::mem::take(plain)));
        }
    };

    while i < chars.len() {
        let c = chars[i];
        match c {
            // 双引号字符串（支持反斜杠转义）
            '"' => {
                flush(&mut out, &mut plain);
                at_line_start = false;
                let mut s = String::from('"');
                i += 1;
                while i < chars.len() {
                    let ch = chars[i];
                    s.push(ch);
                    if ch == '\\' && i + 1 < chars.len() {
                        i += 1;
                        s.push(chars[i]);
                    } else if ch == '"' {
                        break;
                    }
                    i += 1;
                }
                out.push((Tok::Str, s));
            },
            // 单引号字符串（无转义）
            '\'' => {
                flush(&mut out, &mut plain);
                at_line_start = false;
                let mut s = String::from('\'');
                i += 1;
                while i < chars.len() {
                    let ch = chars[i];
                    s.push(ch);
                    if ch == '\'' {
                        break;
                    }
                    i += 1;
                }
                out.push((Tok::Str, s));
            },
            // 行内注释：位于词边界（行首或空白之后）
            '#' if i == 0 || chars[i - 1] == ' ' || chars[i - 1] == '\t' => {
                flush(&mut out, &mut plain);
                out.push((Tok::Comment, chars[i..].iter().collect()));
                return out;
            },
            // 变量：$NAME 或 ${...}
            '$' => {
                flush(&mut out, &mut plain);
                at_line_start = false;
                let mut s = String::from("$");
                i += 1;
                if i < chars.len() && chars[i] == '{' {
                    while i < chars.len() {
                        s.push(chars[i]);
                        if chars[i] == '}' {
                            break;
                        }
                        i += 1;
                    }
                } else {
                    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                        s.push(chars[i]);
                        i += 1;
                    }
                    i -= 1;
                }
                out.push((Tok::Variable, s));
            },
            // 单词：首个非空白词若命中关键字则高亮
            _ if c.is_alphanumeric() || c == '_' || c == '-' => {
                let start = i;
                while i < chars.len()
                    && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '-')
                {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                if at_line_start && KEYWORDS.contains(&word.as_str()) {
                    flush(&mut out, &mut plain);
                    out.push((Tok::Keyword, word));
                } else {
                    plain.push_str(&word);
                }
                at_line_start = false;
                i -= 1; // 循环末尾统一 +1
            },
            _ => {
                if !c.is_whitespace() {
                    at_line_start = false;
                }
                plain.push(c);
            },
        }
        i += 1;
    }
    flush(&mut out, &mut plain);
    out
}

/// 配置文件查看器组件。
///
/// # 属性
/// * `on_close` - 关闭查看器的回调
#[component]
pub fn ConfigViewer(on_close: Callback<()>) -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState should be provided");
    let (content, set_content) = signal(None::<String>);
    let (error, set_error) = signal(None::<String>);

    // 打开时读取配置文件内容
    let _ = Effect::new(move || {
        spawn_local(async move {
            match crate::api::commands::get_config_content().await {
                Ok(c) => set_content.set(Some(c)),
                Err(e) => set_error.set(Some(e.display())),
            }
        });
    });

    view! {
        <div class="modal-overlay" on:click=move |_| on_close.run(())>
            <div class="modal modal--wide" on:click=|e| e.stop_propagation()>
                <div class="modal__header">
                    <div>
                        <h2 class="modal__title">{move || t("config_viewer.title")}</h2>
                        <div class="config-viewer__path">{move || state.config_path.get()}</div>
                    </div>
                    <button class="modal__close" on:click=move |_| on_close.run(())>
                        "✕"
                    </button>
                </div>
                <div class="modal__body">
                    {move || {
                        if let Some(err) = error.get() {
                            return view! { <div class="alert alert--error">{err}</div> }.into_any();
                        }
                        match content.get() {
                            None => view! {
                                <div class="config-viewer__placeholder">{t("config_viewer.loading")}</div>
                            }.into_any(),
                            Some(c) if c.is_empty() => view! {
                                <div class="config-viewer__placeholder">{t("config_viewer.empty")}</div>
                            }.into_any(),
                            Some(c) => {
                                let lines: Vec<Vec<(Tok, String)>> =
                                    c.lines().map(tokenize_line).collect();
                                view! {
                                    <pre class="code-view">
                                        {lines.into_iter().map(|toks| {
                                            view! {
                                                <div class="code-view__line">
                                                    {
                                                        if toks.is_empty() {
                                                            view! { <span>"\u{00A0}"</span> }.into_any()
                                                        } else {
                                                            toks.into_iter().map(|(tok, text)| {
                                                                let cls = tok_class(tok);
                                                                if cls.is_empty() {
                                                                    view! { {text} }.into_any()
                                                                } else {
                                                                    view! { <span class=cls>{text}</span> }.into_any()
                                                                }
                                                            }).collect::<Vec<_>>().into_any()
                                                        }
                                                    }
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </pre>
                                }.into_any()
                            },
                        }
                    }}
                </div>
                <div class="modal__footer">
                    <button class="btn btn--secondary" on:click=move |_| on_close.run(())>
                        {move || t("config_viewer.close")}
                    </button>
                </div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_line_comment() {
        let toks = tokenize_line("# a comment");
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].0, Tok::Comment);
    }

    #[test]
    fn test_alias_line() {
        let toks = tokenize_line("alias gs='git status' # shortcut");
        assert_eq!(toks[0], (Tok::Keyword, "alias".to_string()));
        assert!(toks.iter().any(|(t, s)| *t == Tok::Str && s == "'git status'"));
        assert_eq!(toks.last().unwrap().0, Tok::Comment);
    }

    #[test]
    fn test_double_quoted_string_with_escape() {
        let toks = tokenize_line("export GREETING=\"hi \\\"world\\\"\"");
        assert_eq!(toks[0].0, Tok::Keyword);
        let strs: Vec<&String> = toks.iter().filter(|(t, _)| *t == Tok::Str).map(|(_, s)| s).collect();
        assert_eq!(strs.len(), 1);
        assert!(strs[0].starts_with('"') && strs[0].ends_with('"'));
    }

    #[test]
    fn test_variable_token() {
        // 引号外的变量单独成词；引号内变量按设计归属于字符串词元
        let toks = tokenize_line("source $HOME/.aliases");
        assert_eq!(toks[0].0, Tok::Keyword);
        assert!(toks.iter().any(|(t, s)| *t == Tok::Variable && s == "$HOME"));
    }

    #[test]
    fn test_non_keyword_word_not_highlighted() {
        let toks = tokenize_line("myfunc() { echo hi; }");
        assert!(!toks.iter().any(|(t, _)| *t == Tok::Keyword));
    }
}
