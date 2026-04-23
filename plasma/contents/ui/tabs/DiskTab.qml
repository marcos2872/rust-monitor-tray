import QtQuick 2.15
import QtQuick.Layouts 1.15
import "../components"
import ".."

ColumnLayout {
    id: root

    property var diskMetrics: ({})
    property var diskReadHistory: ({})
    property var diskWriteHistory: ({})
    property real diskReadRate: 0
    property real diskWriteRate: 0
    property int historyDurationMs: 5 * 60 * 1000

    function seriesLength(series) {
        return series && series.count !== undefined ? series.count : 0;
    }

    function seriesValue(series, index) {
        if (!series || !series.buffer || index < 0 || index >= root.seriesLength(series))
            return 0;
        var actualIndex = (series.start + index) % series.buffer.length;
        return Number(series.buffer[actualIndex] || 0);
    }

    function seriesMaximum(series) {
        var maximum = 1;
        for (var i = 0; i < root.seriesLength(series); i += 1)
            maximum = Math.max(maximum, root.seriesValue(series, i));
        return maximum;
    }

    function ioHistoryMaximum() {
        return Math.max(root.seriesMaximum(root.diskReadHistory), root.seriesMaximum(root.diskWriteHistory), 1);
    }

    function historyWindowLabel() {
        return "Últimos " + Math.max(1, Math.round(historyDurationMs / 60000)) + " min";
    }

    function primaryDisk() {
        var rows = root.diskMetrics && root.diskMetrics.disks
            ? root.diskMetrics.disks.slice(0)
            : [];
        if (rows.length === 0) return null;
        rows.sort(function(a, b) {
            if (a.mount_point === "/") return -1;
            if (b.mount_point === "/") return 1;
            return (b.usage_percent || 0) - (a.usage_percent || 0);
        });
        return rows[0];
    }

    function secondaryDisks() {
        var primary = root.cachedPrimaryDisk;
        var rows = root.diskMetrics && root.diskMetrics.disks
            ? root.diskMetrics.disks.slice(0)
            : [];
        rows = rows.filter(function(item) {
            return !primary || item.mount_point !== primary.mount_point;
        });
        rows.sort(function(a, b) { return (b.usage_percent || 0) - (a.usage_percent || 0); });
        return rows.slice(0, 4);
    }

    readonly property var cachedPrimaryDisk: root.diskMetrics ? primaryDisk() : null
    readonly property var cachedSecondaryDisks: root.diskMetrics ? secondaryDisks() : []

    Layout.fillWidth: true
    spacing: theme.spacingM

    MetricCard {
        Layout.fillWidth: true
        hero: true
        title: "Disk"
        subtitle: root.cachedPrimaryDisk
            ? root.cachedPrimaryDisk.name + " · " + root.cachedPrimaryDisk.mount_point
            : "Sem dados"

        HeroMetric {
            Layout.fillWidth: true
            label: "Uso principal"
            value: root.cachedPrimaryDisk ? String(Math.round(root.cachedPrimaryDisk.usage_percent)) : "0"
            unit: "%"
            accentColor: theme.diskColor
            footnote: root.cachedPrimaryDisk
                ? (theme.fmtOne(root.cachedPrimaryDisk.used_space) + " de " + theme.fmtOne(root.cachedPrimaryDisk.total_space) + " GB")
                : "-"
        }

        MetricBar {
            visible: root.cachedPrimaryDisk !== null
            label: root.cachedPrimaryDisk ? root.cachedPrimaryDisk.mount_point : "-"
            value: root.cachedPrimaryDisk ? root.cachedPrimaryDisk.usage_percent : 0
            barColor: theme.diskColor
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.successColor
            label: "Livre"
            value: root.cachedPrimaryDisk ? theme.fmtOne(root.cachedPrimaryDisk.available_space) + " GB" : "-"
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "I/O Activity"
        subtitle: root.historyWindowLabel()

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingM

            HeroMetric {
                Layout.fillWidth: true
                label: "Leitura"
                value: theme.fmtRate(root.diskReadRate)
                accentColor: theme.diskColor
            }

            HeroMetric {
                Layout.fillWidth: true
                label: "Escrita"
                value: theme.fmtRate(root.diskWriteRate)
                accentColor: theme.dangerColor
            }
        }

        SectionHeader {
            title: "Read"
            subtitle: theme.fmtRate(root.diskReadRate)
        }

        HistoryChart {
            Layout.fillWidth: true
            series: root.diskReadHistory
            strokeColor: theme.diskColor
            maximumValue: root.ioHistoryMaximum()
            maxLabel: theme.fmtRate(root.ioHistoryMaximum())
            minLabel: "0 B/s"
        }

        SectionHeader {
            title: "Write"
            subtitle: theme.fmtRate(root.diskWriteRate)
        }

        HistoryChart {
            Layout.fillWidth: true
            series: root.diskWriteHistory
            strokeColor: theme.dangerColor
            maximumValue: root.ioHistoryMaximum()
            maxLabel: theme.fmtRate(root.ioHistoryMaximum())
            minLabel: "0 B/s"
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Partições"
        subtitle: "Secundárias por uso"

        Repeater {
            model: root.cachedSecondaryDisks

            delegate: ColumnLayout {
                Layout.fillWidth: true
                spacing: theme.spacingXS

                MetricBar {
                    Layout.fillWidth: true
                    label: modelData.mount_point
                    value: modelData.usage_percent
                    barColor: theme.diskColor
                    barHeight: 8
                }

                MetricRow {
                    Layout.fillWidth: true
                    dense: true
                    accentColor: "transparent"
                    label: theme.fmtOne(modelData.used_space) + " de " + theme.fmtOne(modelData.total_space) + " GB"
                    value: theme.fmtOne(modelData.available_space) + " GB livres"
                }
            }
        }
    }

    Theme { id: theme }
}
