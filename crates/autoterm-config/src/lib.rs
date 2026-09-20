//! PLAN-025 B 部:AutoTerm 配置文件——默认 shell 与 profiles。
//!
//! 契约(docs/specs/app-unified-entry.md SD-02):
//! - 位置:`AUTOTERM_CONFIG` 环境变量覆盖路径 → 缺省 exe 同目录
//!   `config.toml`(三件套同目录分发契约,003 §5);
//! - Schema V1(TOML):
//!
//!   ```toml
//!   default_profile = "ash"
//!
//!   [[profiles]]
//!   name = "ash"
//!   commandline = 'D:\autostack\auto-shell\ash\target\release\ash.exe'
//!   starting_directory = 'D:\autostack'
//!
//!   [[profiles]]
//!   name = "cmd"
//!   commandline = 'cmd.exe'
//!   ```
//!
//! - 解析失败/文件缺席 = 空 profile 集(消费方回落 COMSPEC/cmd——行为
//!   等价无配置现状,不崩,G6);
//! - 解析优先级在消费方(显式参数 > `--profile` > 配置文件 > COMSPEC);
//! - spawn 参数化经既有 SpawnSpec(`engine_spawn_ex` program/argv/cwd),
//!   `ctrl_c_mode` V1 不进 schema(PLAN-025 §10.3 DEBT)。
//!
//! 依赖纪律:仅 autoterm-ui CLI 与 app-back 消费;autoterm-core 零依赖
//! 面不动(AC-07)。

use std::path::{Path, PathBuf};

/// 一个 profile:名字 + 命令行 + 起始目录。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    /// 完整命令行(program + 参数;经 [`Config::resolve`] 拆分)。
    pub commandline: String,
    /// 起始目录(None/空 = 继承宿主 cwd)。
    pub starting_directory: Option<String>,
}

/// 解析产物。缺席/损坏 → 全默认(空集 + 无 default)。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Config {
    /// default_profile 键指定的名字(未声明 = None)。
    pub default_profile_name: Option<String>,
    /// profile 集(按声明序;重名后者覆盖前者,§5 启动告警语义)。
    pub profiles: Vec<Profile>,
}

/// spawn 三元组(与 `engine_spawn_ex` 的 program/argv/cwd 一一对应;
/// cwd 空 = 继承宿主)。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpawnSpec {
    pub program: String,
    pub argv: Vec<String>,
    pub cwd: String,
}

impl Config {
    /// 按默认搜索路径加载(AUTOTERM_CONFIG → exe 同目录;缺文件/损坏
    /// = 空配置)。
    pub fn load_default() -> Config {
        match default_path() {
            Some(p) => Config::load_from(&p),
            None => Config::default(),
        }
    }

    /// 从指定文件加载。缺席/损坏 = 空配置(不崩);单条 profile 字段
    /// 非法 = 跳过该条并 stderr 告警(一条坏不拖垮全文件)。
    pub fn load_from(path: &Path) -> Config {
        let raw = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => return Config::default(),
        };
        Config::parse(&raw)
    }

    /// 解析配置文本(单测面)。
    pub fn parse(raw: &str) -> Config {
        let value: toml::Value = match toml::from_str(raw) {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "[autoterm-config] ⚠ 配置解析失败,忽略全部 profiles(回落 COMSPEC):{e}"
                );
                return Config::default();
            }
        };
        let default_profile = value
            .get("default_profile")
            .and_then(|v| v.as_str())
            .map(str::to_owned);
        let mut profiles: Vec<Profile> = Vec::new();
        if let Some(items) = value.get("profiles").and_then(|v| v.as_array()) {
            for (i, item) in items.iter().enumerate() {
                let name = item.get("name").and_then(|v| v.as_str());
                let commandline = item.get("commandline").and_then(|v| v.as_str());
                let (Some(name), Some(commandline)) = (name, commandline) else {
                    eprintln!(
                        "[autoterm-config] ⚠ profiles[{i}] 缺 name/commandline,跳过该条"
                    );
                    continue;
                };
                let starting_directory = item
                    .get("starting_directory")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned);
                let profile = Profile {
                    name: name.to_owned(),
                    commandline: commandline.to_owned(),
                    starting_directory,
                };
                // 重名 = 后者覆盖 + 告警(§5)。
                if let Some(slot) = profiles.iter_mut().find(|p| p.name == profile.name) {
                    eprintln!(
                        "[autoterm-config] ⚠ profile 名重复 \"{}\",后者覆盖前者",
                        profile.name
                    );
                    *slot = profile;
                } else {
                    profiles.push(profile);
                }
            }
        }
        if let Some(ref d) = default_profile {
            if !profiles.iter().any(|p| p.name == *d) {
                eprintln!(
                    "[autoterm-config] ⚠ default_profile \"{d}\" 未命中任何 profile,忽略"
                );
            }
        }
        Config { default_profile_name: default_profile, profiles }
    }

    /// 按名取 profile。
    pub fn profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.name == name)
    }

    /// 缺省 profile(default_profile 键;未声明/未命中 = None,消费方
    /// 回落 COMSPEC)。
    pub fn default_profile(&self) -> Option<&Profile> {
        self.default_profile_name.as_deref().and_then(|n| self.profile(n))
    }

    /// profile → spawn 三元组(program/argv/cwd)。
    pub fn spawn_spec(profile: &Profile) -> SpawnSpec {
        let (program, argv) = Config::resolve(&profile.commandline);
        SpawnSpec {
            program,
            argv,
            cwd: profile.starting_directory.clone().unwrap_or_default(),
        }
    }

    /// 命令行拆分:program + argv。规则(V1):
    /// - 双引号开头 → 闭合引号前为 program,其余按空白拆 argv;
    /// - 否则首个空白前为 program,其余按空白拆 argv。
    /// 相对路径/裸名原样透传(由 OS PATH/cwd 解析,与 WT 语义一致)。
    pub fn resolve(commandline: &str) -> (String, Vec<String>) {
        let s = commandline.trim();
        if s.is_empty() {
            return (String::new(), Vec::new());
        }
        if let Some(rest) = s.strip_prefix('"') {
            if let Some(end) = rest.find('"') {
                let program = rest[..end].to_owned();
                let argv = rest[end + 1..]
                    .split_whitespace()
                    .map(str::to_owned)
                    .collect();
                return (program, argv);
            }
        }
        let mut it = s.splitn(2, char::is_whitespace);
        let program = it.next().unwrap_or_default().to_owned();
        let argv = it
            .next()
            .map(|r| r.split_whitespace().map(str::to_owned).collect())
            .unwrap_or_default();
        (program, argv)
    }
}

/// 配置搜索路径:AUTOTERM_CONFIG(非空)→ exe 同目录 config.toml。
/// exe 目录不可得 = None(等价无配置)。
pub fn default_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("AUTOTERM_CONFIG") {
        if !p.trim().is_empty() {
            return Some(PathBuf::from(p));
        }
    }
    let exe = std::env::current_exe().ok()?;
    exe.parent().map(|d| d.join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"
default_profile = "ash"

[[profiles]]
name = "ash"
commandline = 'D:\autostack\auto-shell\ash\target\release\ash.exe'
starting_directory = 'D:\autostack'

[[profiles]]
name = "cmd"
commandline = 'cmd.exe'
"#;

    #[test]
    fn valid_config_parses() {
        let c = Config::parse(VALID);
        assert_eq!(c.default_profile_name.as_deref(), Some("ash"));
        assert_eq!(c.profiles.len(), 2);
        let ash = c.profile("ash").unwrap();
        assert_eq!(
            ash.commandline,
            r"D:\autostack\auto-shell\ash\target\release\ash.exe"
        );
        assert_eq!(ash.starting_directory.as_deref(), Some(r"D:\autostack"));
        let cmd = c.profile("cmd").unwrap();
        assert_eq!(cmd.commandline, "cmd.exe");
        assert_eq!(cmd.starting_directory, None);
        // default 命中声明名。
        assert_eq!(c.default_profile().unwrap().name, "ash");
    }

    #[test]
    fn absent_file_is_empty_config() {
        let c = Config::load_from(Path::new("Z:/definitely/not/here/config.toml"));
        assert_eq!(c, Config::default());
        assert_eq!(c.default_profile(), None);
    }

    #[test]
    fn corrupt_file_is_empty_config() {
        let c = Config::parse("this is [ not = toml {{{");
        assert_eq!(c, Config::default());
    }

    #[test]
    fn profile_missing_fields_skipped() {
        let c = Config::parse(
            r#"
default_profile = "cmd"

[[profiles]]
name = "broken"

[[profiles]]
name = "cmd"
commandline = 'cmd.exe'
"#,
        );
        assert_eq!(c.profiles.len(), 1, "缺 commandline 的条目跳过,不拖垮全文件");
        assert_eq!(c.default_profile().unwrap().name, "cmd");
    }

    #[test]
    fn duplicate_name_latter_wins() {
        let c = Config::parse(
            r#"
[[profiles]]
name = "p"
commandline = 'first.exe'

[[profiles]]
name = "p"
commandline = 'second.exe --flag'
"#,
        );
        assert_eq!(c.profiles.len(), 1);
        assert_eq!(c.profiles[0].commandline, "second.exe --flag");
    }

    #[test]
    fn default_profile_name_unmatched_is_none() {
        let c = Config::parse(
            r#"
default_profile = "ghost"

[[profiles]]
name = "cmd"
commandline = 'cmd.exe'
"#,
        );
        assert_eq!(c.default_profile(), None, "未命中名不得回落首个 profile");
        assert!(c.profile("cmd").is_some(), "按名取不受影响");
    }

    #[test]
    fn resolve_plain_quoted_and_relative() {
        // 裸名 + 无参。
        assert_eq!(
            Config::resolve("cmd.exe"),
            ("cmd.exe".to_owned(), Vec::<String>::new())
        );
        // 裸名 + 参数。
        assert_eq!(
            Config::resolve("pwsh.exe -NoLogo -WorkingDirectory D:\\tmp"),
            (
                "pwsh.exe".to_owned(),
                vec![
                    "-NoLogo".to_owned(),
                    "-WorkingDirectory".to_owned(),
                    "D:\\tmp".to_owned()
                ]
            )
        );
        // 带空格的引号路径 + 参数。
        assert_eq!(
            Config::resolve(r#""D:\Program Files\tool\t.exe" --verbose arg two""#),
            (
                r"D:\Program Files\tool\t.exe".to_owned(),
                vec!["--verbose".to_owned(), "arg".to_owned(), "two\"".to_owned()]
            )
        );
        // 相对路径原样透传(OS 解析)。
        assert_eq!(
            Config::resolve(r".\tools\ash.exe -x"),
            (
                r".\tools\ash.exe".to_owned(),
                vec!["-x".to_owned()]
            )
        );
        // 空串。
        assert_eq!(Config::resolve("  "), (String::new(), Vec::<String>::new()));
    }

    #[test]
    fn spawn_spec_maps_profile_fields() {
        // 带空格路径必须引号包裹(未引号则首空白处拆分——resolve 规则)。
        let p = Profile {
            name: "ash".into(),
            commandline: r#""D:\a b\ash.exe" --fast"#.into(),
            starting_directory: Some(r"D:\autostack".into()),
        };
        let s = Config::spawn_spec(&p);
        assert_eq!(s.program, r"D:\a b\ash.exe");
        assert_eq!(s.argv, vec!["--fast".to_owned()]);
        assert_eq!(s.cwd, r"D:\autostack");
        // 无 starting_directory = 空 cwd(继承宿主)。
        let p2 = Profile { starting_directory: None, ..p };
        assert_eq!(Config::spawn_spec(&p2).cwd, "");
    }

    #[test]
    fn default_path_env_override() {
        // 保存/恢复环境变量(测试串行面)。
        let prev = std::env::var("AUTOTERM_CONFIG").ok();
        unsafe { std::env::set_var("AUTOTERM_CONFIG", r"D:\custom\my.toml") };
        assert_eq!(default_path(), Some(PathBuf::from(r"D:\custom\my.toml")));
        unsafe { std::env::set_var("AUTOTERM_CONFIG", "") };
        assert_ne!(
            default_path(),
            Some(PathBuf::from(r"D:\custom\my.toml")),
            "空 env 值 = 不覆盖,回落 exe 目录"
        );
        match prev {
            Some(v) => unsafe { std::env::set_var("AUTOTERM_CONFIG", v) },
            None => unsafe { std::env::remove_var("AUTOTERM_CONFIG") },
        }
    }

    #[test]
    fn load_from_real_file_roundtrip() {
        let dir = std::env::temp_dir().join("autoterm-config-test");
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("config.toml");
        std::fs::write(&f, VALID).unwrap();
        let c = Config::load_from(&f);
        assert_eq!(c.default_profile().unwrap().name, "ash");
        std::fs::remove_file(&f).ok();
    }
}
