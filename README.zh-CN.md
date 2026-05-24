# atosl-rs

*其他语言版本：[English](README.md)。*

`atosl` 是一个用于本地符号化（symbolication）的 Rust 命令行工具和库。它把原始的二进制地址解析为函数名和源码位置：有 DWARF 调试信息时优先使用 DWARF，缺少调试信息时回退到符号表。

它的设计目标是跨平台工具链、CI 流水线、崩溃日志处理工具，以及任何需要 `atos` 风格符号化、又不想依赖苹果宿主环境的开发流程。

## 为什么有这个项目

苹果的 `atos` 很好用，但它和苹果的运行时环境深度耦合。`atosl` 聚焦于团队在构建系统和工具链中通常真正需要的部分：

- 单个本地可执行文件，外加可嵌入的 Rust API
- 适合脚本处理的输出：`text`、`json`、`json-pretty` 以及流式的 `json-lines`
- DWARF 优先解析，符号表兜底
- 按架构或 UUID 选择 Fat Mach-O 的切片（slice）
- 针对苹果特定行为的可复现回归测试

## 当前的质量基线

- 基于 `clap` v4 的命令行，使用显式的长选项
- 结构化 JSON 与美化 JSON 输出
- 针对解析、UUID 处理、架构别名、地址运算的单元测试
- 构建真实样例二进制、端到端验证符号化的集成测试
- 苹果特定的 Mach-O/DWARF 黄金测试（golden test），带可复现的样例和已签入的快照
- 针对 `--arch` 与 `--uuid` 的 Fat Mach-O 切片选择黄金测试
- 针对苹果单切片与 Fat 二进制流程的 JSON 输出黄金测试
- 针对解析器选择与逐帧查找过程的 verbose 诊断黄金测试
- 在 macOS 上把 `atosl` 与苹果自带 `/usr/bin/atos` 逐帧对拍的差分测试（覆盖 DWARF、默认模式 vs `--inline-frames`，以及 `-l 0`/`-offset` 模式）
- 用于批量符号化吞吐量的 Criterion 基准测试目标
- 覆盖 `fmt`、`clippy`、测试与发布构建的 GitHub Actions CI

## 它擅长处理什么

- 从可执行文件、目标文件（object file）和 dSYM 载荷进行本地符号化
- 通过 `--inline-frames` 展开 DWARF 帧的内联调用栈（等同于 `atos -i`）
- 一次调用解析多个地址
- 地址来源可以是命令行、文件（`--input`）或标准输入（在 `text` 与 `json-lines` 模式下流式输出）
- 支持 `.dSYM` bundle 目录，或按 `--uuid` / build-id 在某个目录中查找
- 通过经 CRC 校验的 `.gnu_debuglink`、build-id 或 debuginfod 缓存查找独立的 ELF 调试文件
- 支持 Mach-O Fat 二进制并显式选择切片
- 通过 JSON 输出进行机器可读的集成
- 通过 verbose 诊断调试符号化决策过程

## 安装

从 crates.io 安装：

```bash
cargo install atosl
```

从源码构建：

```bash
git clone https://github.com/everettjf/atosl-rs.git
cd atosl-rs
cargo build --release
./target/release/atosl --help
```

## 用法

```bash
atosl -o <OBJECT_PATH> -l <LOAD_ADDRESS> [OPTIONS] <ADDRESS>...
```

必填参数：

- `-o, --object <OBJECT_PATH>`：目标文件、可执行文件、dSYM 载荷、`.dSYM` bundle 目录，或一个配合 `--uuid` 进行查找的目录
- `-l, --load-address <LOAD_ADDRESS>`：镜像的运行时加载地址
- `<ADDRESS>...`：要符号化的地址；省略时从 `--input` 或标准输入读取

常用选项：

- `-f, --file-offsets`：直接用 `address − load-address` 作为查找地址，**不**再重定位到 `__TEXT` 的 vmaddr 上。这是为向后兼容保留的历史模式；它与 `atos -offset` **并不等价**（见[地址模式](#地址模式)）。
- `--inline-frames`：把内联函数展开成完整调用栈（最内层在前），等同于 `atos -i`。默认关闭。详见[内联帧](#内联帧)。
- `-a, --arch <ARCH>`：在 Fat 二进制中选择某个 Mach-O 切片
- `--uuid <UUID>`：按 UUID 选择 Mach-O 切片，或按 UUID/build-id 从目录中选择文件
- `-i, --input <FILE>`：从文件读取地址（未给出任何地址时默认读取标准输入）
- `--debug-dir <DIR>`：用于查找独立 ELF 调试文件的额外根目录（可重复指定）
- `--format <text|json|json-pretty|json-lines>`：选择输出格式（`json-lines` 每个地址输出一个 ndjson 对象，在输入流模式下流式输出）
- `-v, --verbose`：把解析器诊断信息打印到标准错误

### 地址模式

`atosl` 会根据你传入的选项来解释每个地址：

| 模式 | 选项 | 查找地址 | 典型用途 |
| --- | --- | --- | --- |
| 加载地址模式（默认） | _无_、`-l <load>` | `address − load_address + __TEXT vmaddr` | 崩溃报告中的运行时/虚拟地址，配合镜像的加载地址 |
| `atos -offset` 等价用法 | `-l 0 <off>` | `off + __TEXT vmaddr` | 相对镜像 `__TEXT` 基址的文件偏移 |
| 文件偏移（遗留 `-f`） | `-f -l <load>` | `address − load_address` | 跳过 `__TEXT` 重定位的向后兼容模式 |

要复现苹果 `atos -offset N`，用默认模式配合零加载地址即可：`atosl -l 0 N` 算出 `N + __TEXT vmaddr`，这正是 `atos -offset` 的行为。`-f` 是一个独立的历史模式，为了不影响老用户而特意保持原样。

## 示例

符号化单个地址：

```bash
atosl -o MyApp.app/MyApp -l 0x100000000 0x100001234
```

符号化多个地址：

```bash
atosl -o MyApp.app/MyApp -l 0x100000000 0x100001234 0x100004321 0x100008888
```

直接指向 `.dSYM` bundle（会自动定位其中的 DWARF 载荷）：

```bash
atosl -o MyApp.app.dSYM -l 0x100000000 0x100001234
```

选择特定的 Fat Mach-O 切片：

```bash
atosl -o Flutter -l 0x100000000 --arch arm64 0x100001234
```

从标准输入读取地址（text 输出逐行流式给出结果）：

```bash
printf '0x100001234\n0x100004321\n' | atosl -o MyApp.app.dSYM -l 0x100000000
```

从文件读取地址：

```bash
atosl -o MyApp.app.dSYM -l 0x100000000 --input crash_addresses.txt
```

在一个存放 dSYM/二进制的目录中按 UUID（或 build-id）查找匹配镜像：

```bash
atosl -o ./symbols -l 0x100000000 --uuid 34FBD46D4A1F3B41A0F14E57D7E25B04 0x100001234
```

输出机器可读的格式：

```bash
atosl -o MyApp.app/MyApp -l 0x100000000 --format json 0x100001234
```

每个地址流式输出一个 JSON 对象（ndjson），例如把崩溃日志中的地址用管道传入：

```bash
cat addresses.txt | atosl -o MyApp.app.dSYM -l 0x100000000 --format json-lines
```

像 `atos -offset 0x4660` 那样符号化一个文件偏移（默认模式 + 零加载地址）：

```bash
atosl -o MyApp.app.dSYM -l 0 0x4660
```

把内联函数展开成完整调用栈（等同于 `atos -i`）：

```bash
atosl -o MyApp.app.dSYM -l 0x100000000 --inline-frames 0x100001234
```

用 verbose 诊断查看解析器的行为：

```bash
atosl -v -o MyApp.app/MyApp -l 0x100000000 --arch arm64 0x100001234
```

JSON 输出形态示例：

```json
{
  "object_path": "MyApp.app/MyApp",
  "object_name": "MyApp",
  "selected_slice": {
    "arch": "arm64",
    "uuid": "34FBD46D-4A1F-3B41-A0F1-4E57D7E25B04"
  },
  "frames": [
    {
      "status": "resolved",
      "requested_address": 4294971956,
      "lookup_address": 4660,
      "symbol": "main",
      "object_name": "MyApp",
      "offset": 0,
      "resolver": "symbol_table",
      "location": {
        "file": "src/main.rs",
        "line": 12
      }
    }
  ]
}
```

## 文本输出

当有 DWARF 源码信息时：

```text
my::function (in MyApp) (src/main.rs:42)
```

当只有符号表时：

```text
my::function (in MyApp) + 16
```

当符号化失败时：

```text
N/A - failed to search symbol table
```

## 内联帧

默认情况下，文本输出只打印最外层的帧——也就是物理上包含该地址的、真正未被内联的函数。这与不带选项的苹果 `atos` 一致（也与早期 `atosl` 版本的输出一致）：

```text
outer (in MyApp) (outer.c:15)
```

加上 `--inline-frames` 即可展开完整的内联调用栈，从最内层的帧开始，与 `atos -i` / `atos --inlineFrames` 的行为相同：

```text
leaf_inline (in MyApp) (helpers.c:5)
mid_inline (in MyApp) (helpers.c:10)
outer (in MyApp) (outer.c:15)
```

JSON 输出不受该开关影响：它始终把最内层的帧作为主结果，并把外层的内联帧列在 `inlined_by` 字段下，因此机器可读的消费者始终能拿到完整的内联信息。

## 作为库使用

`atosl` 除命令行外，也提供库 API：

```rust
use atosl::{atosl, OutputFormat, SymbolizeOptions};

let report = atosl::symbolize_path(&SymbolizeOptions {
    object_path: "fixture_bin".into(),
    load_address: 0,
    addresses: vec![0x1234],
    verbose: false,
    file_offsets: false,
    inline_frames: false,
    arch: None,
    uuid: None,
    format: OutputFormat::Json,
    input: None,
    debug_dirs: Vec::new(),
})?;
```

返回的 `SymbolizeReport` 保留了所选切片、每个地址的解析器选择、查找地址、符号名，以及可选的源码位置。

## 回归测试资产

苹果特定行为由签入到 `tests/golden/apple/` 的黄金文件保护：

- DWARF 支持与符号表支持的 Mach-O 输入的文本输出
- 单切片与 Fat Mach-O 流程的 JSON 输出
- 解析器追踪与切片选择的 verbose 诊断
- 针对有歧义的 Fat 二进制的负路径（失败路径）覆盖

在 macOS 上用以下命令刷新这些快照：

```bash
./scripts/refresh_apple_goldens.sh
```

此外，`tests/atos_differential.rs` 会在宿主机上构建真实的 Mach-O + dSYM，并断言 `atosl` 与苹果 `/usr/bin/atos` 逐帧一致：默认模式对比不带选项的 `atos`、`--inline-frames` 对比 `atos -i`、`atosl -l 0 <off>` 对比 `atos -offset <off>`。这些测试在非 macOS 环境或 `atos` 不可用时会自动跳过，因此在 Linux CI 上是空操作。

## 开发

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release
./scripts/refresh_apple_goldens.sh
cargo bench --bench batch_symbolize
```

只编译基准测试二进制而不执行它（CI 风格的校验）：

```bash
cargo bench --bench batch_symbolize --no-run
```

发布步骤记录在 [RELEASING.md](RELEASING.md)。
若想用一条命令完成发布流程，运行 `./deploy.sh [patch|minor|major|X.Y.Z]`。

## 已知限制

- 它仍不是苹果 `atos` 的 1:1 克隆
- 符号化质量取决于目标二进制中的符号和 DWARF 数据
- Mach-O 工作流仍是主要设计目标；其他目标文件格式在有符号时效果最好
- 测试覆盖了苹果的 UUID 和 dSYM 布局，但真实崩溃日志的解析摄入仍不在范围内
- **不跟随 Mach-O 的调试映射（debug map）**。当你用 `-g` 编译但没有运行 `dsymutil` 时，可执行文件里只保留指向原始 `.o` 文件的 `N_OSO` stab，而行号表 DWARF 存在于那些目标文件中。苹果 `atos` 会顺着这个调试映射去恢复源码行，而 `atosl` 不会，因此对这类二进制只能回退到符号表（`符号 + 偏移`）。要获得源码位置，请让 `atosl` 指向由 `dsymutil` 生成的 `.dSYM`（或本身内嵌 DWARF 的目标文件）。
- 源码路径以完整路径打印。苹果 `atos` 在不加 `-fullPath` 时只打印文件名，而 `atosl` 始终打印 DWARF 行号表中记录的路径。

## 许可证

MIT，详见 `LICENSE`。
