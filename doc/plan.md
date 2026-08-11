# 修改计划：为 audiobook-forge 增加单独生成章节信息文件功能
## 总体思路
当前 `audiobook-forge build` 在合并出 M4B 后会调用 AtomicParsley / MP4Box 把章节元数据写回文件，这一步在你的环境里报错（典型原因是 AtomicParsley 在大文件或特殊标签上 crash，或 MP4Box 缺失）。**可行的修复路径是：复用现有的"按 `/path/to/My Audiobook` 扫描源文件 → 生成章节时间戳"的逻辑，把它从构建流水线里抽出来，新增一个子命令只导出 ffmpeg 风格的章节元数据文件**，再用一条单独的 `ffmpeg -i book.m4b -i chapters.txt -map_metadata 1 -codec copy` 命令把章节回灌进去——这一步不依赖 AtomicParsley/MP4Box，绕开报错点。
下面是改造后的工作流：
```mermaid
flowchart LR
    A["源目录<br/>/path/to/My Audiobook"] --> B{选择路径}
    B -->|现有 build| C["完整流水线<br/>转码+合并+章节注入"]
    C --> D["M4B（含章节）"]
    B -->|新增 export-chapters| E["复用扫描+时间戳逻辑"]
    E --> F["chapters.txt<br/>ffmpeg metadata 格式"]
    F --> G["ffmpeg -i book.m4b -i chapters.txt<br/>-map_metadata 1 -codec copy"]
    G --> H["M4B（章节已回灌）"]
    D -.报错回退.-> F
```
---
## 一、定位现有章节生成逻辑
代码结构为 `src/{audio, cli, core, models, ui, utils}`，章节相关逻辑分散在以下位置（需你按实际确认函数名与行号）：
| 模块 | 作用 | 需要关注的内容 |
|---|---|---|
| `src/core/scanner.rs`（或 `processor.rs`） | 扫描 `--root` 目录、自然排序音轨 | `Scanner::scan_book_folder` 之类返回 `BookFolder` / `Vec<Track>` 的函数 |
| `src/audio/` 下分析模块 | 用 ffprobe 取每条音轨时长 | 取 duration 的工具函数（多半调 `ffprobe -v quiet -print_format json -show_format`）|
| `src/core/` 下章节构造 | 按文件名/CUE/EPUB/Audnex 生成章节列表 | `Chapter { start_ms, end_ms, title }` 结构与对应的 builder |
| `src/cli/` | clap 子命令定义 | `Commands::Build`、`Commands::Metadata(MetadataArgs)` 枚举 |
| 构建主流程 | 合并后写章节 | 调 AtomicParsley / MP4Box 的那一段——**报错点** |
v2.9.0 已经引入了"用 ffprobe 读取已有 M4B 章节"的能力，说明 `src/audio/` 或 `src/metadata/` 下已有一个 `ffprobe` 解析器可直接复用来读回现有章节。
---
## 二、设计新的 CLI 子命令
建议在 `metadata` 下加 `export-chapters` 子命令，与现有 `metadata enrich` 对齐：
```rust
// src/cli/mod.rs （或 commands.rs）
#[derive(Subcommand, Debug)]
pub enum MetadataSub {
    /// 现有的 enrich
    Enrich(MetadataEnrichArgs),
    /// 新增：只导出章节元数据文件，不做任何转码/注入
    ExportChapters(ExportChaptersArgs),
}
#[derive(Parser, Debug)]
pub struct ExportChaptersArgs {
    /// 源目录或已生成的 M4B 文件
    /// 复用 build 的 --root 语义
    #[arg(long, value_name = "PATH")]
    pub root: PathBuf,
    /// 输出文件；不指定则写到 <root>/<book_name>.chapters.txt
    #[arg(long, short = 'o', value_name = "FILE")]
    pub output: Option<PathBuf>,
    /// 章节标题来源：filename / cue / epub / audnex:<ASIN>
    /// 默认 filename，与 build 默认一致
    #[arg(long, default_value = "filename")]
    pub chapters_source: String,
    /// 时间基：1/1000 (默认, ffmpeg 友好) 或 1/44100 等
    #[arg(long, default_value_t = String::from("1/1000"))]
    pub timebase: String,
    /// 同时输出 JSON 版本（便于程序后续处理）
    #[arg(long)]
    pub json: bool,
}
```
调用形式：
```bash
audiobook-forge metadata export-chapters --root "/path/to/My Audiobook"
# → 生成 /path/to/My Audiobook/My Audiobook.chapters.txt
```
---
## 三、抽取并复用章节生成逻辑
把构建流程中"扫描 → 取时长 → 生成章节列表"那一段抽成一个纯函数，返回 `Vec<Chapter>`，供 `build` 和 `export-chapters` 共用：
```rust
// src/core/chapters.rs （新建或扩展已有文件）
use crate::models::{BookFolder, Chapter};
use anyhow::Result;
/// 复用点：根据 root 目录生成章节列表（不依赖任何外部注入工具）
pub async fn build_chapters_from_folder(
    root: &Path,
    source: &ChapterSource,
) -> Result<Vec<Chapter>> {
    // 1. 复用 Scanner：let folder = Scanner::scan_book_folder(root)?;
    // 2. 复用 Analyzer 取每轨 duration（ffprobe）
    // 3. 累加偏移得到 start_ms / end_ms
    // 4. 按 source 解析标题（filename / cue / epub / audnex）
    //    —— 这些在 v2.9.0 的 Chapter Update System 里都已存在
    Ok(chapters)
}
pub struct Chapter {
    pub start_ms: u64,
    pub end_ms: u64,
    pub title: String,
}
```
然后在 `build` 主流程里把原先内联的逻辑替换为 `build_chapters_from_folder(...)` 的调用——这样既加了新功能，又不改变现有行为。
---
## 四、实现章节文件写入
ffmpeg 的章节元数据格式（与 `metadata enrich` 读取的 timestamped 格式一致）：
```
;FFMETADATA1
[CHAPTER]
TIMEBASE=1/1000
START=0
END=4480000
title=Chapter 1 - Intro
[CHAPTER]
TIMEBASE=1/1000
START=4480001
END=38839999
title=Chapter 2 - The Journey
```
写入函数：
```rust
// src/core/chapters.rs
use std::io::Write;
pub fn write_ffmetadata(chapters: &[Chapter], path: &Path, timebase: &str) -> Result<()> {
    let mut f = std::fs::File::create(path)?;
    writeln!(f, ";FFMETADATA1")?;
    for c in chapters {
        writeln!(f, "[CHAPTER]")?;
        writeln!(f, "TIMEBASE={timebase}")?;
        writeln!(f, "START={}", c.start_ms)?;
        writeln!(f, "END={}", c.end_ms)?;
        writeln!(f, "title={}", sanitize_title(&c.title))?;
        writeln!(f)?; // 空行分隔，提升兼容性
    }
    Ok(())
}
fn sanitize_title(s: &str) -> String {
    // ffmpeg metadata 里换行/等号需转义
    s.replace('\n', " ").replace('=', " - ")
}
pub fn write_json(chapters: &[Chapter], path: &Path) -> Result<()> {
    let v: Vec<_> = chapters.iter().map(|c| serde_json::json!({
        "start_ms": c.start_ms, "end_ms": c.end_ms, "title": c.title
    })).collect();
    std::fs::write(path, serde_json::to_string_pretty(&v)?)?;
    Ok(())
}
```
---
## 五、子命令分发实现
在 `main.rs` / `cli` 的 dispatch 里加分支：
```rust
MetadataSub::ExportChapters(args) => {
    let root = args.root.canonicalize()
        .with_context(|| format!("root 不存在: {:?}", args.root))?;
    let source = ChapterSource::parse(&args.chapters_source)?;
    let chapters = build_chapters_from_folder(&root, &source).await?;
    let out = args.output.unwrap_or_else(|| {
        // 复用 build 里推导书名的逻辑：取 root 的文件名
        root.join(format!("{}.chapters.txt",
            root.file_name().unwrap().to_string_lossy()))
    });
    write_ffmetadata(&chapters, &out, &args.timebase)?;
    println!("已写出章节文件: {}", out.display());
    if args.json {
        let json_path = out.with_extension("chapters.json");
        write_json(&chapters, &json_path)?;
        println!("JSON 版本: {}", json_path.display());
    }
    // 打印摘要便于核对
    println!("共 {} 章，总时长 {:.1} 分钟",
        chapters.len(),
        chapters.last().map(|c| c.end_ms as f64 / 60000.0).unwrap_or(0.0));
}
```
---
## 六、手动把章节回灌到 M4B
生成 `chapters.txt` 后，用一条 ffmpeg 命令完成注入（不依赖 AtomicParsley/MP4Box）：
```bash
ffmpeg -i "/path/to/My Audiobook.m4b" \
       -i "/path/to/My Audiobook.chapters.txt" \
       -map_metadata 1 \
       -map_chapters 1 \
       -codec copy \
       "/path/to/My Audiobook.chapters.m4b"
# 验证
ffprobe -i "/path/to/My Audiobook.chapters.m4b" -print_format json -show_chapters
```
关键点：
- `-codec copy` 保证不重新编码、不损失质量。
- `-map_metadata 1` + `-map_chapters 1` 让 ffmpeg 从第二个输入读取章节。
- 输出到新文件，避免原地覆盖出问题时无法回退；确认无误后再 `mv` 覆盖原文件。
如果源是已经生成但缺章节的 M4B，而不是目录，可以再加一个 ffprobe 回退路径，复用 v2.9.0 里"从 M4B 读取章节"的逻辑，把读到的章节直接导出——这样即便没有源目录也能补救已有 M4B。
---
## 七、测试与验证清单
1. **单元测试**：`write_ffmetadata` 对已知 `Vec<Chapter>` 输出做字符串快照比对；`sanitize_title` 覆盖换行、`=`、中文等用例。
2. **集成测试**（用 `assert_cmd`，仓库已依赖）：
   - 准备一个 3 轨的临时目录，跑 `metadata export-chapters --root <tmp>`。
   - 断言输出文件存在、行数 = `1 + 5*N`、`START` 单调递增、最后一章 `END` 等于总时长。
3. **端到端**：
   - `audiobook-forge build --root "/path/to/My Audiobook"`（复现报错，得到无章节 M4B）。
   - `audiobook-forge metadata export-chapters --root "/path/to/My Audiobook"`。
   - `ffmpeg -i ... -map_chapters 1 -codec copy ...`。
   - `ffprobe ... -show_chapters` 确认章节数与标题正确。
   - 在 Apple Books / Audiobookshelf 里打开，确认可跳章。
4. **回归**：原来的 `build` 流程行为不变（因为只是把内联逻辑抽成函数调用）。
5. **`cargo fmt && cargo clippy && cargo test`** 全绿后再提交。
---
## 八、常见坑与排查
- **ffprobe 时长单位**：ffprobe 返回的 `duration` 是秒（浮点），乘 1000 得毫秒；注意有些容器返回 `N/A`，要回退到 `ffprobe -show_packets -select_streams a` 累加。
- **首章 START 必须 = 0**：Apple Books 对非零起点章节识别不稳定，把第一章 `START` 强制设为 0。
- **TIMEBASE 选择**：`1/1000`（毫秒）最通用；若用 `1/44100` 需保证 START/END 都是整数采样数。
- **标题编码**：ffmpeg metadata 默认 UTF-8，但 Windows 控制台重定向可能写 GBK——`std::fs::File` 直接写 UTF-8 字节即可，不要走 `println!` 再重定向。
- **`-map_metadata 1` 覆盖原有 tag**：会丢失 M4B 里已有的 title/artist 等。如果原文件已有元数据，改用 `ffmpeg -i in.m4b -i chapters.txt -map_metadata 0 -map_chapters 1 -codec copy out.m4b`（metadata 取自输入 0，章节取自输入 1）。
- **AtomicParsley crash 的根因**：通常是封面图过大或 ID3v2.4 标签含特殊字符；即使后续不依赖它，也建议在 issue 里附上 `RUST_LOG=debug audiobook-forge build ... 2>log.txt` 的尾部日志，便于上游修复。
---
## 九、可选增强
- 在 `build` 报错时自动落盘 `chapters.txt` 到输出目录，并打印上面那条 ffmpeg 命令——把"故障兜底"变成内置行为。
- 给 `export-chapters` 加 `--from-m4b` 标志，直接对已有 M4B 用 ffprobe 读章节再导出，省去保留源目录的麻烦。
- 支持 CUE sheet 输出（`--format cue`），方便其他工具链。
- 把章节文件路径写进 `ProcessingResult`，让 YAML 配置 `chapters.export_on_failure: true` 控制是否自动触发。
按以上 1–8 步走，即可在不破坏现有 `build` 行为的前提下，新增一个独立、可复用、绕开注入工具报错的章节导出能力。

