/// 内置模板库数据。
///
/// 提供一组按分类整理的常用 Shell 别名模板。
use crate::models::template::{Template, TemplateCategory};

/// 返回所有内置模板。
pub fn get_builtin_templates() -> Vec<Template> {
    let mut templates = Vec::new();

    // Git 模板
    templates.extend(git_templates());

    // Docker 模板
    templates.extend(docker_templates());

    // 文件操作模板
    templates.extend(file_ops_templates());

    // 网络模板
    templates.extend(network_templates());

    templates
}

/// 返回 Git 相关的别名模板。
fn git_templates() -> Vec<Template> {
    vec![
        Template::with_tags(
            "gs",
            "git status",
            "查看仓库状态",
            TemplateCategory::Git,
            vec!["git".to_string(), "status".to_string()],
        ),
        Template::with_tags(
            "gl",
            "git log --oneline --graph --decorate",
            "查看精简提交日志",
            TemplateCategory::Git,
            vec!["git".to_string(), "log".to_string()],
        ),
        Template::with_tags(
            "ga",
            "git add",
            "添加文件到暂存区",
            TemplateCategory::Git,
            vec!["git".to_string(), "add".to_string()],
        ),
        Template::with_tags(
            "gaa",
            "git add --all",
            "添加所有文件到暂存区",
            TemplateCategory::Git,
            vec!["git".to_string(), "add".to_string()],
        ),
        Template::with_tags(
            "gc",
            "git commit",
            "提交更改",
            TemplateCategory::Git,
            vec!["git".to_string(), "commit".to_string()],
        ),
        Template::with_tags(
            "gcm",
            "git commit -m",
            "提交更改并附消息",
            TemplateCategory::Git,
            vec!["git".to_string(), "commit".to_string()],
        ),
        Template::with_tags(
            "gp",
            "git push",
            "推送到远程仓库",
            TemplateCategory::Git,
            vec!["git".to_string(), "push".to_string()],
        ),
        Template::with_tags(
            "gpl",
            "git pull",
            "从远程仓库拉取",
            TemplateCategory::Git,
            vec!["git".to_string(), "pull".to_string()],
        ),
        Template::with_tags(
            "gd",
            "git diff",
            "查看差异",
            TemplateCategory::Git,
            vec!["git".to_string(), "diff".to_string()],
        ),
        Template::with_tags(
            "gb",
            "git branch",
            "查看/管理分支",
            TemplateCategory::Git,
            vec!["git".to_string(), "branch".to_string()],
        ),
        Template::with_tags(
            "gco",
            "git checkout",
            "切换分支",
            TemplateCategory::Git,
            vec!["git".to_string(), "checkout".to_string()],
        ),
        Template::with_tags(
            "gst",
            "git stash",
            "暂存当前修改",
            TemplateCategory::Git,
            vec!["git".to_string(), "stash".to_string()],
        ),
    ]
}

/// 返回 Docker 相关的别名模板。
fn docker_templates() -> Vec<Template> {
    vec![
        Template::with_tags(
            "dps",
            "docker ps",
            "列出运行中的容器",
            TemplateCategory::Docker,
            vec!["docker".to_string(), "ps".to_string()],
        ),
        Template::with_tags(
            "dpsa",
            "docker ps -a",
            "列出所有容器",
            TemplateCategory::Docker,
            vec!["docker".to_string(), "ps".to_string()],
        ),
        Template::with_tags(
            "di",
            "docker images",
            "列出镜像",
            TemplateCategory::Docker,
            vec!["docker".to_string(), "images".to_string()],
        ),
        Template::with_tags(
            "drm",
            "docker rm",
            "删除容器",
            TemplateCategory::Docker,
            vec!["docker".to_string(), "rm".to_string()],
        ),
        Template::with_tags(
            "drmi",
            "docker rmi",
            "删除镜像",
            TemplateCategory::Docker,
            vec!["docker".to_string(), "rmi".to_string()],
        ),
        Template::with_tags(
            "dex",
            "docker exec -it",
            "进入容器交互终端",
            TemplateCategory::Docker,
            vec!["docker".to_string(), "exec".to_string()],
        ),
        Template::with_tags(
            "dlog",
            "docker logs -f",
            "查看容器日志（跟踪）",
            TemplateCategory::Docker,
            vec!["docker".to_string(), "logs".to_string()],
        ),
        Template::with_tags(
            "dstop",
            "docker stop",
            "停止容器",
            TemplateCategory::Docker,
            vec!["docker".to_string(), "stop".to_string()],
        ),
        Template::with_tags(
            "dcom",
            "docker-compose",
            "Docker Compose 命令",
            TemplateCategory::Docker,
            vec!["docker".to_string(), "compose".to_string()],
        ),
        Template::with_tags(
            "dup",
            "docker-compose up -d",
            "启动 Docker Compose 服务（后台）",
            TemplateCategory::Docker,
            vec!["docker".to_string(), "compose".to_string()],
        ),
    ]
}

/// 返回文件操作相关的别名模板。
fn file_ops_templates() -> Vec<Template> {
    vec![
        Template::with_tags(
            "ll",
            "ls -la",
            "详细列出文件（含隐藏文件）",
            TemplateCategory::FileOps,
            vec!["ls".to_string(), "list".to_string()],
        ),
        Template::with_tags(
            "la",
            "ls -A",
            "列出文件（含隐藏，不含 . 和 ..）",
            TemplateCategory::FileOps,
            vec!["ls".to_string(), "list".to_string()],
        ),
        Template::with_tags(
            "lt",
            "ls -ltr",
            "按修改时间排序列出文件",
            TemplateCategory::FileOps,
            vec!["ls".to_string(), "list".to_string()],
        ),
        Template::with_tags(
            "lsize",
            "ls -lhS",
            "按文件大小排序列出",
            TemplateCategory::FileOps,
            vec!["ls".to_string(), "list".to_string()],
        ),
        Template::with_tags(
            "du1",
            "du -h --max-depth=1",
            "查看目录大小（一级深度）",
            TemplateCategory::FileOps,
            vec!["du".to_string(), "disk".to_string()],
        ),
        Template::with_tags(
            "dfh",
            "df -h",
            "查看磁盘使用情况",
            TemplateCategory::FileOps,
            vec!["df".to_string(), "disk".to_string()],
        ),
        Template::with_tags(
            "mkdirp",
            "mkdir -p",
            "递归创建目录",
            TemplateCategory::FileOps,
            vec!["mkdir".to_string()],
        ),
        Template::with_tags(
            "cpv",
            "cp -rv",
            "递归复制（显示进度）",
            TemplateCategory::FileOps,
            vec!["cp".to_string()],
        ),
        Template::with_tags(
            "rmrf",
            "rm -rf",
            "强制递归删除",
            TemplateCategory::FileOps,
            vec!["rm".to_string()],
        ),
    ]
}

/// 返回网络相关的别名模板。
fn network_templates() -> Vec<Template> {
    vec![
        Template::with_tags(
            "ping5",
            "ping -c 5",
            "Ping 5 次",
            TemplateCategory::Network,
            vec!["ping".to_string()],
        ),
        Template::with_tags(
            "ports",
            "netstat -tulanp",
            "查看监听端口",
            TemplateCategory::Network,
            vec!["netstat".to_string(), "port".to_string()],
        ),
        Template::with_tags(
            "myip",
            "curl ifconfig.me",
            "查看公网 IP",
            TemplateCategory::Network,
            vec!["curl".to_string(), "ip".to_string()],
        ),
        Template::with_tags(
            "localip",
            "ipconfig getifaddr en0",
            "查看局域网 IP (macOS)",
            TemplateCategory::Network,
            vec!["ip".to_string()],
        ),
        Template::with_tags(
            "curlhead",
            "curl -I",
            "查看 HTTP 响应头",
            TemplateCategory::Network,
            vec!["curl".to_string(), "http".to_string()],
        ),
        Template::with_tags(
            "wgetm",
            "wget --mirror",
            "镜像下载网站",
            TemplateCategory::Network,
            vec!["wget".to_string()],
        ),
    ]
}
