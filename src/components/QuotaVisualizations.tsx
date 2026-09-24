import { useEffect, useRef, useState } from "react";
import type {
  QuotaAnalytics,
  QuotaDailyPoint,
  QuotaVisualizationConfig,
} from "../types/config";

interface VizTheme {
  accent: string;
  subText: string;
  mainText: string;
  emptyCell: string;
  scheme?: "light" | "dark";
}

/** GitHub contribution-graph greens (Primer success scale). */
const GITHUB_HEAT_LIGHT = ["#ebedf0", "#9be9a8", "#40c463", "#30a14e", "#216e39"] as const;
const GITHUB_HEAT_DARK = ["#161b22", "#0e4429", "#006d32", "#26a641", "#39d353"] as const;

function isLightScheme(theme: VizTheme): boolean {
  if (theme.scheme) return theme.scheme === "light";
  const rgb = parseColorRgb(theme.mainText);
  if (!rgb) return false;
  return (rgb[0] * 299 + rgb[1] * 587 + rgb[2] * 114) / 1000 < 128;
}

function githubHeatPalette(theme: VizTheme): readonly string[] {
  return isLightScheme(theme) ? GITHUB_HEAT_LIGHT : GITHUB_HEAT_DARK;
}

function formatUsdFromCents(cents: number): string {
  return `$${(cents / 100).toFixed(2)}`;
}

function formatCompactTokens(value: number): string {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}K`;
  return String(Math.round(value));
}

function parseColorRgb(color: string): [number, number, number] | null {
  const trimmed = color.trim();
  if (trimmed.startsWith("#")) {
    const hex = trimmed.slice(1);
    if (hex.length === 6) {
      return [
        parseInt(hex.slice(0, 2), 16),
        parseInt(hex.slice(2, 4), 16),
        parseInt(hex.slice(4, 6), 16),
      ];
    }
    if (hex.length === 3) {
      return [
        parseInt(hex[0] + hex[0], 16),
        parseInt(hex[1] + hex[1], 16),
        parseInt(hex[2] + hex[2], 16),
      ];
    }
  }
  const rgbMatch = trimmed.match(
    /rgba?\(\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)/i,
  );
  if (rgbMatch) {
    return [
      Math.round(Number(rgbMatch[1])),
      Math.round(Number(rgbMatch[2])),
      Math.round(Number(rgbMatch[3])),
    ];
  }
  return null;
}

/** Sqrt-scaled heat level used by GitHub-style contribution greens. */
function heatLevel(value: number, max: number): 0 | 1 | 2 | 3 | 4 {
  if (value <= 0 || max <= 0) return 0;
  const t = Math.sqrt(Math.min(1, value / max));
  if (t >= 0.85) return 4;
  if (t >= 0.6) return 3;
  if (t >= 0.35) return 2;
  return 1;
}

function formatHeatValue(value: number, isCost: boolean): string {
  if (isCost) return formatUsdFromCents(value);
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(2)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}K`;
  return value.toFixed(value % 1 ? 1 : 0);
}

function metricIsTokens(
  primary?: "queries" | "tokens" | null,
  legacy?: "queries" | "tokens" | null,
): boolean {
  return (primary ?? legacy) === "tokens";
}

function pickDailySeries(
  analytics: QuotaAnalytics,
  useTokens: boolean,
): QuotaDailyPoint[] {
  if (useTokens && (analytics.daily_tokens?.length ?? 0) > 0) {
    return analytics.daily_tokens!;
  }
  if (!useTokens && (analytics.daily_queries?.length ?? 0) > 0) {
    return analytics.daily_queries!;
  }
  return analytics.daily_activity ?? [];
}

function modelMetricIsTokens(config: QuotaVisualizationConfig): boolean {
  return config.model_metric === "tokens";
}

const HEATMAP_ROWS = 5;
const HEATMAP_CELL_TARGET_PX = 8;
const HEATMAP_CELL_MAX_PX = 11;
const HEATMAP_CELL_GAP_PX = 2;
const DAILY_BAR_MIN_PX = 6;
const DAILY_BAR_GAP_PX = 3;
const DAILY_BAR_MIN_COUNT = 7;

function useElementWidth<T extends HTMLElement>() {
  const ref = useRef<T | null>(null);
  const [width, setWidth] = useState(0);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const update = () => setWidth(el.clientWidth);
    update();
    const observer = new ResizeObserver(update);
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  return { ref, width };
}

function fitCount(width: number, minSize: number, gap: number, maxCount: number): number {
  if (maxCount <= 0) return 0;
  if (width <= 0) return Math.min(DAILY_BAR_MIN_COUNT, maxCount);
  const fitted = Math.floor((width + gap) / (minSize + gap));
  return Math.max(1, Math.min(maxCount, fitted));
}

function fitCellSize(width: number, count: number, gap: number, minSize: number, maxSize: number): number {
  if (count <= 0) return minSize;
  if (width <= 0) return minSize;
  return Math.min(
    maxSize,
    Math.max(minSize, (width - gap * Math.max(0, count - 1)) / count),
  );
}

function CalendarHeatmapViz({
  points,
  theme,
  valueLabel,
  valueIsCost = false,
}: {
  points: QuotaDailyPoint[];
  theme: VizTheme;
  valueLabel: string;
  valueIsCost?: boolean;
}) {
  const { ref: gridRef, width: gridWidth } = useElementWidth<HTMLDivElement>();

  const cols = Math.max(1, fitCount(gridWidth, HEATMAP_CELL_TARGET_PX, HEATMAP_CELL_GAP_PX, 64));
  const capacity = cols * HEATMAP_ROWS;
  const recent = points.slice(-capacity);
  const leadingEmpty = Math.max(0, capacity - recent.length);
  const cells: Array<QuotaDailyPoint | null> = [
    ...Array.from({ length: leadingEmpty }, () => null),
    ...recent,
  ];
  const max = Math.max(...recent.map((p) => p.value), 0);
  const cellSize = fitCellSize(
    gridWidth,
    cols,
    HEATMAP_CELL_GAP_PX,
    HEATMAP_CELL_TARGET_PX,
    HEATMAP_CELL_MAX_PX,
  );
  const palette = githubHeatPalette(theme);
  const emptyOutline = isLightScheme(theme)
    ? "inset 0 0 0 1px rgba(27, 31, 35, 0.06)"
    : "inset 0 0 0 1px rgba(240, 246, 252, 0.08)";

  return (
    <div className="mt-1.5 flex flex-col gap-1 w-full">
      <div className="flex justify-between items-center gap-2">
        <span
          className="text-[7px] font-bold uppercase tracking-wider opacity-60"
          style={{ color: theme.subText }}
        >
          {valueLabel}
        </span>
        <div className="flex items-center gap-1 flex-shrink-0">
          <span className="text-[6px] opacity-40" style={{ color: theme.subText }}>
            Less
          </span>
          {[0, 1, 2, 3, 4].map((level) => (
            <div
              key={level}
              className="rounded-[2px] flex-shrink-0"
              style={{
                width: 8,
                height: 8,
                backgroundColor: palette[level],
                boxShadow: level === 0 ? emptyOutline : undefined,
              }}
            />
          ))}
          <span className="text-[6px] opacity-40" style={{ color: theme.subText }}>
            More
          </span>
        </div>
      </div>
      <div
        ref={gridRef}
        className="w-full"
        style={{
          display: "grid",
          gridAutoFlow: "column",
          gridTemplateRows: `repeat(${HEATMAP_ROWS}, ${cellSize}px)`,
          gridTemplateColumns: `repeat(${cols}, ${cellSize}px)`,
          gap: HEATMAP_CELL_GAP_PX,
        }}
      >
        {cells.map((day, index) => {
          const level = day ? heatLevel(day.value, max) : 0;
          return (
            <div
              key={day?.date ?? `empty-${index}`}
              title={
                day
                  ? `${day.date}: ${formatHeatValue(day.value, valueIsCost)}`
                  : undefined
              }
              className="rounded-[2px]"
              style={{
                width: cellSize,
                height: cellSize,
                backgroundColor: palette[level],
                boxShadow: level === 0 ? emptyOutline : undefined,
              }}
            />
          );
        })}
      </div>
    </div>
  );
}

function DailyBarsViz({
  points,
  theme,
  valueLabel,
  valueIsCost = false,
}: {
  points: QuotaDailyPoint[];
  theme: VizTheme;
  valueLabel: string;
  valueIsCost?: boolean;
}) {
  const { ref: barsRef, width: barsWidth } = useElementWidth<HTMLDivElement>();
  const barCount = Math.max(
    DAILY_BAR_MIN_COUNT,
    fitCount(barsWidth, DAILY_BAR_MIN_PX, DAILY_BAR_GAP_PX, points.length || DAILY_BAR_MIN_COUNT),
  );
  const recent = points.slice(-Math.min(barCount, Math.max(points.length, 1)));
  const max = Math.max(...recent.map((p) => p.value), 0);
  const palette = githubHeatPalette(theme);

  return (
    <div className="mt-1.5 flex flex-col gap-1 w-full">
      <span
        className="text-[7px] font-bold uppercase tracking-wider opacity-60"
        style={{ color: theme.subText }}
      >
        {valueLabel} · {recent.length}d
      </span>
      <div
        ref={barsRef}
        className="flex items-end w-full"
        style={{ height: 40, gap: DAILY_BAR_GAP_PX }}
      >
        {recent.map((day) => {
          const level = heatLevel(day.value, max);
          const h =
            day.value > 0 && max > 0
              ? Math.max(12, Math.sqrt(day.value / max) * 100)
              : 6;
          return (
            <div
              key={day.date}
              title={`${day.date.slice(5)}: ${formatHeatValue(day.value, valueIsCost)}`}
              className="flex-1 min-w-0 rounded-t-sm transition-all"
              style={{
                height: `${h}%`,
                backgroundColor: palette[level],
              }}
            />
          );
        })}
      </div>
    </div>
  );
}

function ModelBreakdownViz({
  models,
  theme,
  useTokens,
}: {
  models: NonNullable<QuotaAnalytics["model_breakdown"]>;
  theme: VizTheme;
  useTokens: boolean;
}) {
  const rows = models
    .map((m) => ({
      ...m,
      displayValue: useTokens ? (m.token_total ?? m.value) : m.value,
    }))
    .sort((a, b) => b.displayValue - a.displayValue);
  const max = Math.max(...rows.map((m) => m.displayValue), 1);

  return (
    <div className="mt-1.5 flex flex-col gap-1">
      <span
        className="text-[7px] font-bold uppercase tracking-wider opacity-60"
        style={{ color: theme.subText }}
      >
        {useTokens ? "By model (tokens)" : "By model ($)"}
      </span>
      {rows.slice(0, 6).map((m) => (
        <div key={m.model} className="flex flex-col gap-0.5">
          <div className="flex justify-between text-[7px] font-semibold gap-1">
            <span className="truncate" style={{ color: theme.subText }} title={m.model}>
              {m.model}
            </span>
            <span className="tabular-nums flex-shrink-0" style={{ color: theme.mainText }}>
              {useTokens
                ? formatCompactTokens(m.displayValue)
                : formatUsdFromCents(m.displayValue)}
            </span>
          </div>
          <div className="w-full h-0.5 bg-white/5 rounded-full overflow-hidden">
            <div
              className="h-full rounded-full"
              style={{
                width: `${(m.displayValue / max) * 100}%`,
                backgroundColor: theme.accent,
              }}
            />
          </div>
        </div>
      ))}
    </div>
  );
}

export function QuotaVisualizations({
  provider,
  analytics,
  config,
  theme,
}: {
  provider: string;
  analytics?: QuotaAnalytics | null;
  config?: QuotaVisualizationConfig | null;
  theme: VizTheme;
}) {
  if (!analytics || !quotaVisualizationsEnabled(config)) {
    return null;
  }

  const isClaude = provider === "claude-code";
  if (provider !== "cursor" && !isClaude) return null;

  const cfg = config!;
  const heatmapUsesTokens = metricIsTokens(cfg.heatmap_metric, cfg.daily_metric);
  const barsUsesTokens = metricIsTokens(cfg.bars_metric, cfg.daily_metric);
  const heatmapSeries = pickDailySeries(analytics, heatmapUsesTokens);
  const barsSeries = pickDailySeries(analytics, barsUsesTokens);
  const hasModels = (analytics.model_breakdown?.length ?? 0) > 0;

  return (
    <div className="border-t border-white/5 pt-1.5 mt-1 flex flex-col gap-0.5 w-full">
      {cfg.calendar_heatmap && heatmapSeries.length > 0 && (
        <CalendarHeatmapViz
          points={heatmapSeries}
          theme={theme}
          valueLabel={heatmapUsesTokens ? "Daily tokens" : "Daily queries"}
        />
      )}
      {cfg.daily_bars && barsSeries.length > 0 && (
        <DailyBarsViz
          points={barsSeries}
          theme={theme}
          valueLabel={barsUsesTokens ? "Daily tokens" : "Daily queries"}
        />
      )}
      {cfg.model_breakdown && hasModels && (
        <ModelBreakdownViz
          models={analytics.model_breakdown!}
          theme={theme}
          useTokens={modelMetricIsTokens(cfg) || isClaude}
        />
      )}
    </div>
  );
}

export const DEFAULT_QUOTA_VISUALIZATIONS: QuotaVisualizationConfig = {
  calendar_heatmap: false,
  daily_bars: false,
  model_breakdown: false,
  heatmap_metric: "queries",
  bars_metric: "queries",
  model_metric: "spend",
};

export function quotaVisualizationsEnabled(config?: QuotaVisualizationConfig | null): boolean {
  if (!config) return false;
  return !!(
    config.calendar_heatmap ||
    config.daily_bars ||
    config.model_breakdown
  );
}

export function providerSupportsQuotaVisualizations(provider?: string | null): boolean {
  return provider === "cursor" || provider === "claude-code";
}
