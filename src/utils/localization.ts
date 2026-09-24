import { useEffect } from "react";
import { isMacOS } from "./platform";

export type UiLanguage = "zh-CN" | "en";

export function resolveLanguage(language?: string): UiLanguage {
  return language === "zh-CN" || (language !== "en" && isMacOS) ? "zh-CN" : "en";
}

// The existing interface contains labels in many independent widgets. Keep the
// source copy intact so the language can be changed live in every Tauri window.
const ZH: Record<string, string> = {
  "Overview": "总览", "Dashboard": "主界面", "Settings": "设置", "About": "关于",
  "General": "通用", "General Settings": "通用设置", "Sidebar": "侧边栏",
  "Sidebar Settings": "侧边栏设置", "Quota Monitor": "额度监控",
  "Quota monitor settings": "额度监控设置", "Quota monitor options for this software": "此软件的额度监控选项",
  "Use a local IDE login, or paste an API key when the provider supports it.": "使用本地 IDE 登录；服务商支持时也可填写 API 密钥。",
  "Agent & API Quotas": "Agent 与 API 额度", "GPU Monitor": "GPU 监控",
  "GPU Monitor Status": "GPU 监控状态", "Paper Deadlines": "论文截止日期",
  "Deadlines": "截止日期", "Arxiv Radar": "arXiv 雷达",
  "Quick Launch Widgets": "独立浮窗", "Independent Widgets": "独立浮窗", "Open Sidebar": "打开侧边栏",
  "Interface Language": "界面语言",
  "Change the language in the dashboard, sidebar, and widgets.": "切换主界面、侧边栏和浮窗的显示语言。",
  "The sidebar groups all modules in one place. Open independent floating widgets only when needed; use the menu bar icon to reopen this window or the sidebar.": "侧边栏集中显示各模块；需要时再打开独立浮窗。菜单栏图标可以重新打开主界面或侧边栏。",
  "Track AI agent & API limits on your desktop": "在桌面查看 AI Agent 与 API 额度",
  "Floating desktop monitoring for GPU clusters": "在桌面监控 GPU 集群",
  "Track conference deadlines on your desktop": "在桌面查看会议截止日期",
  "Swipe to discover latest research papers": "滑动浏览最新研究论文",
  "GPU Monitor Widget": "GPU 监控浮窗", "Paper Deadlines Widget": "论文截止日期浮窗",
  "Arxiv Radar Widget": "arXiv 雷达浮窗", "Quota Monitor Widget": "额度监控浮窗",
  "Hide All Widgets": "隐藏全部浮窗", "Hide Widget": "隐藏浮窗",
  "Show Widget": "显示浮窗", "Active": "已显示", "Ready": "未显示",
  "Total GPUs": "GPU 总数", "Active Deadlines": "待截止日期",
  "ALL SERVERS OFFLINE": "全部服务器离线", "All servers offline": "全部服务器离线",
  "Refresh failed": "刷新失败", "Refresh failed · cached": "刷新失败 · 显示缓存",
  "Update failed": "更新失败", "Update failed · cached": "更新失败 · 显示缓存",
  "Update failed — showing cached GPU data.": "更新失败，正在显示缓存的 GPU 数据。",
  "Refresh failed — showing cached GPU data.": "刷新失败，正在显示缓存的 GPU 数据。",
  "Update failed — showing cached deadlines.": "更新失败，正在显示缓存的截止日期。",
  "Refresh failed — showing cached deadlines.": "刷新失败，正在显示缓存的截止日期。",
  "Update failed — showing cached papers.": "更新失败，正在显示缓存的论文。",
  "Refresh failed — showing cached papers.": "刷新失败，正在显示缓存的论文。",
  "Update failed — showing cached quota data.": "更新失败，正在显示缓存的额度数据。",
  "Refresh failed — showing cached quota data.": "刷新失败，正在显示缓存的额度数据。",
  "7d Usage": "7 天用量",
  "Monitored Agents": "监控的 Agent", "No Sidebar Widgets": "侧边栏暂无模块",
  "Open Settings": "打开设置", "No Agents Configured": "尚未配置 Agent",
  "No Servers Configured": "尚未配置服务器", "No Conferences Tracked": "尚未关注会议",
  "No active data. Configure servers in Settings.": "暂无数据，请在设置中配置服务器。",
  "No agents configured. Go to Settings to add one.": "尚未配置 Agent，请到设置中添加。",
  "No deadlines match your current filters.": "当前筛选条件下没有截止日期。",
  "Open Settings → GPU Monitor to add SSH hosts.": "到设置 → GPU 监控中添加 SSH 主机。",
  "Open the main window Settings → Quota Monitor to add providers.": "到主窗口的设置 → 额度监控中添加服务商。",
  "Adjust filters in Settings → Paper Deadlines.": "到设置 → 论文截止日期中调整筛选条件。",
  "Enable Arxiv Radar in the dashboard.": "请在主界面启用 arXiv 雷达。",
  "Enable GPU Monitor in the dashboard.": "请在主界面启用 GPU 监控。",
  "Enable Paper Deadlines in the dashboard.": "请在主界面启用论文截止日期。",
  "Enable Quota Monitor in the dashboard.": "请在主界面启用额度监控。",
  "All caught up!": "已全部查看", "Check back later for new papers in CS.": "稍后再来看新的计算机科学论文。",
  "Latest": "最新", "Saved": "已收藏", "Discarded": "已忽略",
  "All": "全部", "All keywords": "全部关键词", "Back": "返回", "More": "更多", "Less": "收起",
  "New": "新增", "Open": "打开", "Close": "关闭", "Delete": "删除", "Delete?": "确认删除？",
  "Copy": "复制", "Copied": "已复制", "Edit": "编辑", "Save": "保存", "Reset": "重置",
  "Refresh All": "全部刷新", "Refresh Deadlines": "刷新截止日期",
  "Refresh Quotas": "刷新额度", "Refresh all GPU servers": "刷新所有 GPU 服务器",
  "Refresh arxiv papers": "刷新 arXiv 论文", "Refresh paper deadlines": "刷新论文截止日期",
  "Refresh papers": "刷新论文", "Refresh quotas": "刷新额度",
  "Restart GPU workers": "重启 GPU 监控", "Service Enabled": "服务已启用",
  "Service Disabled": "服务已关闭", "Enabled": "已启用", "Disabled": "已关闭",
  "Yes": "是", "No": "否", "Online": "在线", "Offline": "离线", "Offline · cached": "离线 · 缓存",
  "Waiting for backend...": "正在等待后台服务…", "No jobs in queue": "队列中没有任务",
  "My jobs": "我的任务", "All users": "所有用户", "Mine": "我的",
  "Queue": "队列", "Memory": "显存", "Temp": "温度", "Job Run Time": "任务运行时间",
  "Open PDF ↑": "打开 PDF ↑", "Open paper": "打开论文", "Remove from saved": "取消收藏",
  "Delete permanently": "永久删除", "← Discard": "← 忽略", "Save →": "收藏 →",
  "Dashboard Theme": "主界面主题",
  "Choose between light and dark mode for the control panel.": "选择主界面的浅色或深色外观。",
  "Light": "浅色", "Dark": "深色", "Widget & Sidebar Scale": "浮窗与侧边栏缩放",
  "Widget and sidebar scale": "浮窗与侧边栏缩放",
  "Scales desktop widgets and the sidebar to match your display. Dashboard always stays at 100%.": "调整浮窗和侧边栏大小以适应屏幕，主界面始终保持 100%。",
  "Launch at Startup": "登录时启动", "Automatically start Widgitron when you log in to Windows.": "登录系统时自动启动 Widgitron。",
  "Automatically start Widgitron when you log in.": "登录系统时自动启动 Widgitron。",
  "Hide Dashboard on Startup": "启动时隐藏主界面",
  "Keep the control panel hidden in the system tray when the app starts.": "应用启动后仅显示在菜单栏。",
  "App Diagnostic Logs": "应用诊断日志",
  "Open the log folder to view runtime logs and troubleshoot issues.": "打开日志文件夹以查看运行记录和排查问题。",
  "Open Log Folder": "打开日志文件夹", "Dock Edge": "停靠边缘",
  "Reveal Sidebar": "打开侧边栏", "Slide the docked widget hub into view.": "显示汇集各模块的侧边栏。",
  "Choose an edge here, or drag the sidebar header near any screen edge to snap it there.": "选择停靠边缘，或拖动侧边栏顶部靠近屏幕边缘。",
  "Choose the screen edge where the sidebar opens.": "选择侧边栏打开时所靠近的屏幕边缘。",
  "Left": "左", "Top": "上", "Right": "右", "Bottom": "下",
  "Pin Display": "保持显示", "Keep the sidebar pinned open instead of hiding when the pointer leaves.": "让侧边栏保持打开。",
  "Open the sidebar automatically when Widgitron starts.": "Widgitron 启动时自动打开侧边栏。",
  "Reveal sensitivity": "唤出灵敏度", "Hide sensitivity": "收起灵敏度",
  "How easily the edge opens the sidebar. Default (4) is conservative.": "调整触碰屏幕边缘时侧边栏的唤出灵敏度。",
  "How quickly it collapses after the pointer leaves. Default (8) is snappy.": "调整指针离开后的收起速度。",
  "Pin Shortcut": "固定快捷键", "Click the recorder, then press a shortcut to toggle pinned display from anywhere.": "点击录制器，再按快捷键切换固定显示。",
  "Press shortcut...": "请按快捷键…", "Esc cancels": "Esc 取消", "Record": "录制",
  "Sidebar Widgets": "侧边栏模块", "Choose which modules appear inside the summonable sidebar.": "选择在侧边栏中显示的模块。",
  "Hide Widget Headers": "隐藏模块标题",
  "Hide each widget's icon, title, status, and refresh row for a denser layout.": "隐藏图标、标题、状态和刷新栏，让布局更紧凑。",
  "Sidebar Theme": "侧边栏主题", "Surface": "背景", "Header": "标题栏",
  "Surface opacity": "背景不透明度", "Header opacity": "标题栏不透明度",
  "Card opacity": "卡片不透明度", "Frosted blur": "磨砂模糊",
  "Changes update the sidebar immediately.": "更改会立即应用到侧边栏。",
  "Theme & Styling": "主题与样式", "Widget Themes": "浮窗主题",
  "Assign Theme to Widgets": "为浮窗分配主题", "Create Theme": "创建主题",
  "Duplicate Theme": "复制主题", "Editing Custom Theme": "编辑自定义主题",
  "Duplicate": "复制", "Built-in": "内置", "Night": "深色",
  "The focused dark sidebar look.": "高对比度深色侧边栏。",
  "A softly translucent white frosted-glass surface.": "高可读性的浅色磨砂侧边栏。",
  "Quota Default": "额度默认", "Quota Transparent": "额度透明",
  "GPU Default": "GPU 默认", "GPU Transparent": "GPU 透明",
  "Deadline Default": "截止日期默认", "Deadline Transparent": "截止日期透明",
  "Arxiv Radar Default": "arXiv 默认", "Arxiv Transparent": "arXiv 透明",
  "This theme is a read-only system preset. Click": "这是只读的系统预设。点击",
  "to customize colors for this widget.": "即可为这个浮窗自定义颜色。",
  "Use": "使用", "on a built-in theme to make an editable personal version.": "复制内置主题后可编辑个人版本。",
  "System Preset": "系统预设", "Preset": "预设", "Unassigned": "未分配",
  "Background": "背景", "Background Color": "背景颜色",
  "Background & Text Colors": "背景与文字颜色", "Color Configuration": "颜色配置",
  "Main Text": "主文字", "Sub Text": "次要文字", "Main Text Color": "主文字颜色",
  "Sub Text Color": "次要文字颜色", "Primary Colors": "主色调",
  "Theme Name": "主题名称", "Opacity": "不透明度",
  "This theme is read-only. You can assign it to widgets below.": "此主题为只读预设，可以分配给下方浮窗。",
  "Start from a built-in look, then copy it to create an editable personal theme.": "选择内置样式后复制，即可创建可编辑的个人主题。",
  "Add New Server": "添加服务器", "Host / IP": "主机名 / IP",
  "Username": "用户名", "Password": "密码", "Port": "端口",
  "Use ~/.ssh/config": "使用 ~/.ssh/config", "Slurm Cluster Mode": "Slurm 集群模式",
  "Enables job-based monitoring via squeue & srun": "通过 squeue 和 srun 监控任务。",
  "Show squeue list": "显示 squeue 列表", "Toggle squeue task list": "切换 squeue 任务列表",
  "Compact Style": "紧凑样式", "Target CCF Ranks": "目标 CCF 等级",
  "Target CORE Ranks": "目标 CORE 等级", "Subscribed Conferences": "关注的会议",
  "Research Category": "研究类别", "Research Edition": "研究版本",
  "Keywords (Comma separated)": "关键词（用逗号分隔）",
  "Arxiv Proxy": "arXiv 代理",
  "Optional proxy URL used only when fetching arxiv papers. Leave blank to connect directly.": "只在抓取 arXiv 论文时使用；留空则直连。",
  "Add New Quota Monitor": "添加额度监控", "Authentication": "认证方式",
  "Local": "本地登录", "API Key": "API 密钥", "API URL": "API 地址",
  "Custom Endpoint": "自定义端点", "Show Account Name": "显示账号名称",
  "Display email or account name in the widget": "在浮窗中显示邮箱或账号名称。",
  "Show Plan Type": "显示套餐类型", "Display plan or subscription tier in the widget": "在浮窗中显示套餐或订阅级别。",
  "Show Interaction Hints": "显示操作提示",
  "Display swipe instructions at the bottom of cards": "在卡片底部显示滑动操作提示。",
  "Analytics Visualizations": "额度图表", "Update Interval (Hours)": "更新间隔（小时）",
  "Max Quota": "额度上限", "Edit directly": "直接编辑", "Add 1 count": "增加 1 次",
  "Use 1 count": "使用 1 次", "Current version": "当前版本",
  "Software Update": "软件更新", "Check for Updates": "检查更新",
  "Update Available": "有可用更新", "Download and Install Update": "下载并安装更新",
  "Retry Download": "重试下载", "Downloading...": "正在下载…",
  "Download completed. Launching installer...": "下载完成，正在启动安装程序…",
  "Config Directory": "配置目录", "Corrupt Config Backups": "损坏配置备份",
  "Open Config Folder": "打开配置文件夹", "Copy Job ID": "复制任务 ID",
  "Close sidebar": "关闭侧边栏", "Drag sidebar": "拖动侧边栏",
  "Pin sidebar open": "启动时打开侧边栏", "Unpin sidebar": "取消启动时打开侧边栏",
  "Unpin sidebar and enable auto-hide": "取消固定并启用自动收起",
  "Restore Position": "恢复位置", "Unlock to move": "解锁后拖动",
  "Resize widget": "调整浮窗大小",
  "Lock position": "锁定位置", "Keep above other windows": "置于其他窗口之上",
  "Keep in normal window order": "恢复普通窗口层级", "Hide widget": "隐藏浮窗",
};

function translated(source: string): string {
  const whitespace = source.match(/^(\s*)([\s\S]*?)(\s*)$/);
  if (!whitespace) return source;
  const [, before, core, after] = whitespace;
  if (ZH[core]) return before + ZH[core] + after;
  const count = core.match(/^(Latest|Saved|Discarded) \((\d+)\)$/);
  if (count) return `${before}${ZH[count[1]]} (${count[2]})${after}`;
  const online = core.match(/^(\d+)\/(\d+) servers online$/i);
  if (online) return `${before}${online[1]}/${online[2]} 台服务器在线${after}`;
  const stale = core.match(/^(\d+) showing cached$/i);
  if (stale) return `${before}${stale[1]} 项显示缓存${after}`;
  const errors = core.match(/^(\d+) update errors?$/i);
  if (errors) return `${before}${errors[1]} 项更新失败${after}`;
  const backoff = core.match(/^Backing off (\d+)s$/i);
  if (backoff) return `${before}${backoff[1]} 秒后重试${after}`;
  return source;
}

type TextRecord = { source: string; rendered: string };
const textRecords = new WeakMap<Text, TextRecord>();
const attributeRecords = new WeakMap<Element, Map<string, TextRecord>>();
const ATTRIBUTES = ["title", "aria-label", "placeholder"] as const;

function localizeText(node: Text, language: UiLanguage) {
  if (node.parentElement?.closest("script,style,textarea,code,pre,[data-no-localize]")) return;
  const previous = textRecords.get(node);
  const source = previous && node.data === previous.rendered ? previous.source : node.data;
  const rendered = language === "zh-CN" ? translated(source) : source;
  textRecords.set(node, { source, rendered });
  if (node.data !== rendered) node.data = rendered;
}

function localizeAttributes(element: Element, language: UiLanguage) {
  const records = attributeRecords.get(element) ?? new Map<string, TextRecord>();
  for (const name of ATTRIBUTES) {
    const value = element.getAttribute(name);
    if (value === null) continue;
    const previous = records.get(name);
    const source = previous && value === previous.rendered ? previous.source : value;
    const rendered = language === "zh-CN" ? translated(source) : source;
    records.set(name, { source, rendered });
    if (value !== rendered) element.setAttribute(name, rendered);
  }
  attributeRecords.set(element, records);
}

function localizeTree(root: Node, language: UiLanguage) {
  if (root.nodeType === Node.TEXT_NODE) {
    localizeText(root as Text, language);
    return;
  }
  if (!(root instanceof Element)) return;
  localizeAttributes(root, language);
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT | NodeFilter.SHOW_TEXT);
  while (walker.nextNode()) {
    const node = walker.currentNode;
    if (node.nodeType === Node.TEXT_NODE) localizeText(node as Text, language);
    else localizeAttributes(node as Element, language);
  }
}

export function useUiLocalization(language: UiLanguage) {
  useEffect(() => {
    document.documentElement.lang = language;
    localizeTree(document.body, language);
    const observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        if (mutation.type === "characterData") localizeTree(mutation.target, language);
        else if (mutation.type === "attributes") localizeAttributes(mutation.target as Element, language);
        else mutation.addedNodes.forEach((node) => localizeTree(node, language));
      }
    });
    observer.observe(document.body, {
      subtree: true,
      childList: true,
      characterData: true,
      attributes: true,
      attributeFilter: [...ATTRIBUTES],
    });
    return () => observer.disconnect();
  }, [language]);
}
