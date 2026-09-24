import Foundation
import SwiftUI
import WidgetKit

private let appGroup = "group.com.evan.widgitron"
private let widgetKind = "com.evan.widgitron.quota"

private struct QuotaSnapshot: Decodable {
    let language: String
    let showPlanType: Bool
    let items: [QuotaItem]

    enum CodingKeys: String, CodingKey {
        case language, items
        case showPlanType = "show_plan_type"
    }
}

private struct QuotaItem: Decodable {
    let name: String
    let provider: String
    let currentValue: Double?
    let maxQuota: Double?
    let unit: String?
    let primaryName: String?
    let primaryReset: String?
    let lastUpdate: String?
    let planType: String?
    let errorMessage: String?

    enum CodingKeys: String, CodingKey {
        case name, provider, unit
        case currentValue = "current_value"
        case maxQuota = "max_quota"
        case primaryName = "primary_name"
        case primaryReset = "primary_reset"
        case lastUpdate = "last_update"
        case planType = "plan_type"
        case errorMessage = "error_msg"
    }

    var progress: Double? {
        guard let currentValue, let maxQuota, maxQuota > 0 else { return nil }
        return min(max(currentValue / maxQuota, 0), 1)
    }

    var amount: String? {
        guard let currentValue else { return nil }
        if unit == "%" { return "\(Int(currentValue.rounded()))%" }
        let number = currentValue.formatted(.number.precision(.fractionLength(0...1)))
        return [number, unit].compactMap { $0 }.joined(separator: " ")
    }

    func usageLabel(chinese: Bool) -> String? {
        guard let primaryName, !primaryName.isEmpty else { return nil }
        if chinese && primaryName == "7d Usage" { return "7 天用量" }
        return primaryName
    }
}

private struct QuotaEntry: TimelineEntry {
    let date: Date
    let snapshot: QuotaSnapshot?
}

private struct QuotaProvider: TimelineProvider {
    func placeholder(in context: Context) -> QuotaEntry {
        QuotaEntry(date: .now, snapshot: QuotaSnapshot(
            language: "zh-CN", showPlanType: true,
            items: [QuotaItem(name: "Codex", provider: "codex", currentValue: 55,
                              maxQuota: 100, unit: "%", primaryName: "7d Usage",
                              primaryReset: nil, lastUpdate: nil,
                              planType: "Pro", errorMessage: nil)]))
    }

    func getSnapshot(in context: Context, completion: @escaping (QuotaEntry) -> Void) {
        completion(context.isPreview ? placeholder(in: context) : currentEntry())
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<QuotaEntry>) -> Void) {
        let entry = currentEntry()
        completion(Timeline(entries: [entry], policy: .after(entry.date.addingTimeInterval(5 * 60))))
    }

    private func currentEntry() -> QuotaEntry {
        guard let directory = FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: appGroup),
              let data = try? Data(contentsOf: directory.appendingPathComponent("quota-snapshot.json")),
              let snapshot = try? JSONDecoder().decode(QuotaSnapshot.self, from: data) else {
            return QuotaEntry(date: .now, snapshot: nil)
        }
        return QuotaEntry(date: .now, snapshot: snapshot)
    }
}

private struct QuotaView: View {
    let entry: QuotaEntry
    @Environment(\.widgetFamily) private var family

    private var chinese: Bool { entry.snapshot?.language != "en" }
    private var visibleItems: [QuotaItem] {
        Array((entry.snapshot?.items ?? []).prefix(family == .systemSmall ? 1 : 3))
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "gauge.with.dots.needle.33percent")
                    .foregroundStyle(.tint)
                Text(chinese ? "额度监控" : "Quota Monitor")
                    .font(.headline)
                Spacer(minLength: 0)
            }
            if visibleItems.isEmpty {
                Spacer(minLength: 0)
                Text(chinese ? "打开 Widgitron 添加额度来源" : "Open Widgitron to add a quota source")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Spacer(minLength: 0)
            } else {
                ForEach(visibleItems.indices, id: \.self) { index in
                    quotaRow(visibleItems[index])
                }
                Spacer(minLength: 0)
            }
        }
        .padding(16)
        .containerBackground(for: .widget) { Color(nsColor: .windowBackgroundColor) }
    }

    private func quotaRow(_ item: QuotaItem) -> some View {
        VStack(alignment: .leading, spacing: 5) {
            HStack(spacing: 6) {
                Text(item.name).font(.subheadline.weight(.semibold)).lineLimit(1)
                if entry.snapshot?.showPlanType == true, let plan = item.planType, !plan.isEmpty {
                    Text(plan.uppercased()).font(.caption2).foregroundStyle(.secondary).lineLimit(1)
                }
                Spacer(minLength: 4)
                if let amount = item.amount {
                    Text(amount).font(.subheadline.weight(.bold)).foregroundStyle(.tint)
                }
            }
            if let progress = item.progress {
                ProgressView(value: progress).tint(.mint)
            } else if let error = item.errorMessage, !error.isEmpty {
                Text(error).font(.caption2).foregroundStyle(.secondary).lineLimit(1)
            } else {
                Text(chinese ? "等待更新" : "Waiting for update")
                    .font(.caption2).foregroundStyle(.secondary)
            }
            if let label = item.usageLabel(chinese: chinese) {
                HStack(spacing: 4) {
                    Text(label)
                    if let reset = item.primaryReset, !reset.isEmpty {
                        Text("(\(reset))")
                    }
                }
                .font(.caption2)
                .foregroundStyle(.secondary)
                .lineLimit(1)
            }
        }
    }
}

private struct QuotaWidget: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(kind: widgetKind, provider: QuotaProvider()) { entry in
            QuotaView(entry: entry)
        }
        .configurationDisplayName("Widgitron · Quota")
        .description("在桌面查看额度使用情况 / View quota usage on your desktop")
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}

@main
struct WidgitronWidgets: WidgetBundle {
    var body: some Widget { QuotaWidget() }
}
