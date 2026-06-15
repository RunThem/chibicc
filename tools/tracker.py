#!/usr/bin/env python3
"""
tracker.py -- chibicc 子仓库 commit 进度追踪脚本
==================================================
用途:在逐 commit 复刻 chibicc 子仓库时, 记录当前复刻到哪个 commit,
     支持在多台电脑间通过 git 同步进度.

命令:
    python tools/tracker.py init  [目录名]     初始化,切到子仓库的第一个 commit
    python tools/tracker.py curr              恢复到记录中的 commit
    python tools/tracker.py next              前进到下一个 commit
    python tools/tracker.py prev              回退到上一个 commit

进度文件 commit-record 为 INI 格式,位于仓库根目录:
    [chibicc-c]
    path = chibicc-c
    commit = 0522e2d77e3ab82d3b80a5be8dbbdc8d4180561c
"""

import configparser
import os
import subprocess
import sys


# ============================================================
# 全局常量
# ============================================================

# 脚本自身所在目录(tools/)
_SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

# 仓库根目录(tools/ 的上一级)
_REPO_ROOT = os.path.dirname(_SCRIPT_DIR)

# 进度记录文件路径(位于仓库根目录)
RECORD_FILE = os.path.join(_REPO_ROOT, "commit-record")


# ============================================================
# 工具函数
# ============================================================

def run_git(dir_path, *args, capture=True):
    """
    在指定目录中执行 git 命令.

    参数:
        dir_path: git 仓库路径
        *args:    传递给 git 子命令的参数,如 'rev-list', '--reverse', 'HEAD'
        capture:  是否捕获并返回 stdout(默认 True)

    返回:
        CompletedProcess 对象;如果命令失败则打印错误并退出.
    """
    cmd = ["git", "-C", dir_path] + list(args)
    result = subprocess.run(cmd, capture_output=capture, text=True)
    if result.returncode != 0:
        print(f"[ERROR] git 命令执行失败: {' '.join(cmd)}")
        if result.stderr:
            print(result.stderr.strip())
        sys.exit(1)
    return result


def get_rev_list(dir_path):
    """
    获取子仓库按时间升序(最早到最新)排列的所有 commit hash 列表.

    每次调用都实时执行 git rev-list,不缓存,确保总是最新状态.

    参数:
        dir_path: 子仓库目录路径

    返回:
        list[str]: commit hash 列表,索引 0 为最早的 commit
    """
    # 用 main 分支而非 HEAD,因为 HEAD 可能处于 detached 状态
    # detached HEAD 下 rev-list HEAD 只会列出当前 commit 及其祖先,导致列表不完整
    result = run_git(dir_path, "rev-list", "--reverse", "main")
    commits = result.stdout.strip().split("\n")
    # 过滤空行(如仓库无 commit)
    return [c for c in commits if c]


def read_record():
    """
    从 commit-record 文件中读取子仓库路径和当前 commit hash.

    返回:
        (dir_path, commit_hash) 元组.
        如果文件或 section 不存在,返回 (None, None).

    commit-record 文件格式(INI):
        [chibicc-c]
        path = chibicc-c
        commit = 0522e2d77e3ab82d3b80a5be8dbbdc8d4180561c
    """
    if not os.path.exists(RECORD_FILE):
        return None, None

    config = configparser.ConfigParser()
    config.read(RECORD_FILE, encoding="utf-8")

    # 只有一个 section,取第一个即可
    sections = config.sections()
    if not sections:
        return None, None

    section = sections[0]
    dir_path = config.get(section, "path", fallback=None)
    commit = config.get(section, "commit", fallback=None)

    # path 可能是相对路径,统一解析为相对于仓库根目录的绝对路径
    if dir_path and not os.path.isabs(dir_path):
        dir_path = os.path.normpath(os.path.join(_REPO_ROOT, dir_path))

    return dir_path, commit


def write_record(dir_path, commit_hash, section_name=None):
    """
    将子仓库路径和当前 commit hash 写入 commit-record 文件.

    参数:
        dir_path:      子仓库目录路径
        commit_hash:   当前 commit 的完整 hash
        section_name:  INI section 名(默认取 dir_path 的最后一级目录名)
    """
    # 计算相对于仓库根目录的路径,便于跨电脑同步
    rel_path = os.path.relpath(dir_path, _REPO_ROOT)

    if section_name is None:
        section_name = os.path.basename(dir_path)

    config = configparser.ConfigParser()
    config[section_name] = {
        "path": rel_path,
        "commit": commit_hash,
    }

    with open(RECORD_FILE, "w", encoding="utf-8") as f:
        config.write(f)

    print(f"[OK] 进度已记录: [{section_name}] path={rel_path}, commit={commit_hash[:7]}")


def checkout(dir_path, commit_hash):
    """
    将子仓库的工作区切换到指定的 commit.

    参数:
        dir_path:    子仓库目录路径
        commit_hash: 要切换到的 commit hash
    """
    run_git(dir_path, "checkout", commit_hash, capture=False)


def show_commit(dir_path, commit_hash):
    """
    打印指定 commit 的简要信息(hash + 标题).

    参数:
        dir_path:    子仓库目录路径
        commit_hash: 要查看的 commit hash
    """
    result = run_git(dir_path, "log", "--oneline", "-1", commit_hash)
    print(result.stdout.strip())


def find_position(commit_hash, commits):
    """
    在 commit 列表中查找指定 hash 的位置(从 1 开始计数).

    参数:
        commit_hash: 要查找的 commit hash
        commits:     commit hash 列表(按时间升序)

    返回:
        int: 位置编号(1-based).未找到时返回 -1.

    注意:hash 比较使用完整长度,不缩写.
    """
    for i, h in enumerate(commits):
        if h == commit_hash:
            return i + 1  # 转为人类友好的 1-based 编号
    return -1


# ============================================================
# 命令处理函数
# ============================================================

def cmd_init(dir_name="chibicc-c"):
    """
    init 命令:将子仓库切到第一个 commit,并记录进度.

    逻辑流程:
        1. 检查 commit-record 是否已存在,若存在则提示跳过
        2. 获取子仓库的全部 commit(升序)
        3. 取出第一个 commit 的 hash
        4. checkout 到该 commit
        5. 写入 commit-record

    参数:
        dir_name: 子仓库目录名或路径(相对于仓库根目录,默认为 chibicc-c)
    """
    # 将 dir_name 解析为绝对路径(相对于仓库根目录)
    if os.path.isabs(dir_name):
        dir_path = dir_name
    else:
        dir_path = os.path.normpath(os.path.join(_REPO_ROOT, dir_name))

    print(f"[DIR] 子仓库路径: {dir_path}")

    # 如果 commit-record 已存在(非空),提示并跳过
    existing_path, existing_commit = read_record()
    if existing_path and existing_commit:
        print(f"[WARN] commit-record 已存在,当前进度:")
        print(f"   路径:   {existing_path}")
        print(f"   commit: {existing_commit[:7]}")
        print("   如需重新初始化,请先手动删除 commit-record 文件.")
        return

    # 获取全部 commit 升序列表
    commits = get_rev_list(dir_path)
    if not commits:
        print(f"[ERROR] 仓库 {dir_path} 中没有找到任何 commit.")
        sys.exit(1)

    # 第一个 commit 即最早的 commit
    first_commit = commits[0]

    print(f"[INFO] 切换到第一个 commit: {first_commit[:7]}")
    checkout(dir_path, first_commit)

    # 写入记录
    write_record(dir_path, first_commit)

    print(f"\n[INFO] 进度: 第 1 / {len(commits)} 个 commit")
    show_commit(dir_path, first_commit)


def cmd_curr():
    """
    curr 命令:恢复到 commit-record 中记录的 commit.

    逻辑流程:
        1. 读取 commit-record,获取 dir_path 和 commit_hash
        2. 检查子仓库目录是否存在
        3. checkout 到记录的 commit
        4. 获取 commit 列表,找到当前位置并显示
    """
    dir_path, commit_hash = read_record()

    if not dir_path or not commit_hash:
        print("[ERROR] 尚未初始化,请先运行 init 命令.")
        print(f"   用法: python tools/tracker.py init [目录名]")
        sys.exit(1)

    # 检查子仓库目录是否存在
    if not os.path.isdir(dir_path):
        print(f"[ERROR] 子仓库目录不存在: {dir_path}")
        print(f"   请确认 commit-record 中的 path 是否正确.")
        sys.exit(1)

    print(f"[DIR] 子仓库路径: {dir_path}")
    print(f"[INFO] 切换到记录中的 commit: {commit_hash[:7]}")
    checkout(dir_path, commit_hash)

    # 获取 commit 列表,计算当前位置
    commits = get_rev_list(dir_path)
    pos = find_position(commit_hash, commits)

    print(f"\n[INFO] 进度: 第 {pos} / {len(commits)} 个 commit")
    show_commit(dir_path, commit_hash)


def cmd_next():
    """
    next 命令:切换到下一个 commit,并更新记录.

    逻辑流程:
        1. 读取 commit-record,获取当前进度
        2. 获取子仓库的全部 commit 列表
        3. 找到当前 commit 在列表中的位置
        4. 如果已是最后一个 commit,提示并退出
        5. 取下一个 commit,checkout,更新记录
    """
    dir_path, commit_hash = read_record()

    if not dir_path or not commit_hash:
        print("[ERROR] 尚未初始化,请先运行 init 命令.")
        sys.exit(1)

    if not os.path.isdir(dir_path):
        print(f"[ERROR] 子仓库目录不存在: {dir_path}")
        sys.exit(1)

    commits = get_rev_list(dir_path)
    pos = find_position(commit_hash, commits)

    if pos == -1:
        print(f"[ERROR] 当前记录的 commit {commit_hash[:7]} 在仓库中未找到.")
        print(f"   可能是子仓库被 rebase 或 force push 过.")
        print(f"   建议重新运行 init.")
        sys.exit(1)

    if pos >= len(commits):
        print(f"[OK] 已是最新的 commit,没有更多了.")
        print(f"   当前: 第 {pos} / {len(commits)} 个")
        show_commit(dir_path, commit_hash)
        return

    # 下一个 commit 在列表中的索引(0-based)
    next_hash = commits[pos]  # pos 是 1-based,列表索引恰好 = pos

    print(f"[INFO] 前进一个 commit:")
    print(f"   {commit_hash[:7]}  ->  {next_hash[:7]}")
    checkout(dir_path, next_hash)
    write_record(dir_path, next_hash)

    print(f"\n[INFO] 进度: 第 {pos + 1} / {len(commits)} 个 commit")
    show_commit(dir_path, next_hash)


def cmd_prev():
    """
    prev 命令:回退到上一个 commit,并更新记录.

    逻辑流程:
        1. 读取 commit-record,获取当前进度
        2. 获取子仓库的全部 commit 列表
        3. 找到当前 commit 在列表中的位置
        4. 如果已是第一个 commit,提示并退出
        5. 取上一个 commit,checkout,更新记录
    """
    dir_path, commit_hash = read_record()

    if not dir_path or not commit_hash:
        print("[ERROR] 尚未初始化,请先运行 init 命令.")
        sys.exit(1)

    if not os.path.isdir(dir_path):
        print(f"[ERROR] 子仓库目录不存在: {dir_path}")
        sys.exit(1)

    commits = get_rev_list(dir_path)
    pos = find_position(commit_hash, commits)

    if pos == -1:
        print(f"[ERROR] 当前记录的 commit {commit_hash[:7]} 在仓库中未找到.")
        print(f"   可能是子仓库被 rebase 或 force push 过.")
        print(f"   建议重新运行 init.")
        sys.exit(1)

    if pos <= 1:
        print(f"[OK] 已是最早的 commit,无法再回退.")
        print(f"   当前: 第 1 / {len(commits)} 个")
        show_commit(dir_path, commit_hash)
        return

    # 上一个 commit 在列表中的索引(0-based)
    prev_hash = commits[pos - 2]  # pos 是 1-based,上一个索引 = pos - 2

    print(f"[INFO] 回退一个 commit:")
    print(f"   {commit_hash[:7]}  <-  {prev_hash[:7]}")
    checkout(dir_path, prev_hash)
    write_record(dir_path, prev_hash)

    print(f"\n[INFO] 进度: 第 {pos - 1} / {len(commits)} 个 commit")
    show_commit(dir_path, prev_hash)


# ============================================================
# 命令行入口
# ============================================================

def main():
    """
    解析命令行参数,分发到对应的命令处理函数.

    支持的子命令:
        init  [dir]   初始化,可选指定子仓库目录,默认为 chibicc-c
        curr          恢复到记录中的 commit
        next          前进到下一个 commit
        prev          回退到上一个 commit
    """
    args = sys.argv[1:]

    if len(args) == 0:
        _print_usage()
        sys.exit(0)

    cmd = args[0]

    if cmd == "init":
        # init [dir_name]
        dir_name = args[1] if len(args) > 1 else "chibicc-c"
        cmd_init(dir_name)

    elif cmd == "curr":
        cmd_curr()

    elif cmd == "next":
        cmd_next()

    elif cmd == "prev":
        cmd_prev()

    elif cmd in ("-h", "--help", "help"):
        _print_usage()

    else:
        print(f"[ERROR] 未知命令: {cmd}")
        _print_usage()
        sys.exit(1)


def _print_usage():
    """打印使用说明."""
    print("""
tracker.py -- chibicc 子仓库 commit 进度追踪工具
====================================================

用法:
    python tools/tracker.py <命令> [参数]

命令:
    init  [目录]   初始化,切到子仓库的第一个 commit
                   目录默认为 chibicc-c,可指定其他子仓库路径
    curr           恢复到记录中保存的 commit
    next           前进到下一个 commit(按时间升序)
    prev           回退到上一个 commit(按时间升序)

示例:
    python tools/tracker.py init              # 初始化,追踪 chibicc-c
    python tools/tracker.py init my-fork      # 初始化,追踪 my-fork 目录
    python tools/tracker.py curr              # 恢复到记录位置
    python tools/tracker.py next              # 前进一个 commit
    python tools/tracker.py prev              # 回退一个 commit

进度记录:
    进度保存在仓库根目录的 commit-record 文件中(INI 格式),
    将该文件 commit 到 git 仓库后,可在多台电脑间同步进度.
""")


if __name__ == "__main__":
    main()
