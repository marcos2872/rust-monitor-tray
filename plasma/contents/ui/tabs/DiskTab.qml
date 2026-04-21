import QtQuick 2.15
import QtQuick.Layouts 1.15
import "../components"
import ".."

ColumnLayout {
    id: root

    property var metrics: ({})
    property var diskReadHistory: []
    property var diskWriteHistory: []
    property real diskReadRate: 0
    property real diskWriteRate: 0

    function fmtOne(value) {
        if (value === undefined || value === null || isNaN(value)) return "0.0";
        return Number(value).toFixed(1);
    }

    function fmtRate(value) {
        if (value === undefined || value === null || isNaN(value)) return "0 B/s";
        var units = ["B", "KB", "MB", "GB"];
        var size  = Number(value);
        var index = 0;
        while (size >= 1024 && index < units.length - 1) {
            size  /= 1024;
            index += 1;
        }
        return size.toFixed(index === 0 ? 0 : 1) + " " + units[index] + "/s";
    }

    function ioHistoryMaximum() {
        var maximum = 1;
        var i;
        for (i = 0; i < diskReadHistory.length; i += 1) {
            maximum = Math.max(maximum, Number(diskReadHistory[i]) || 0);
        }
        for (i = 0; i < diskWriteHistory.length; i += 1) {
            maximum = Math.max(maximum, Number(diskWriteHistory[i]) || 0);
        }
        return maximum;
    }

    function primaryDisk() {
        var rows = metrics && metrics.disk && metrics.disk.disks
            ? metrics.disk.disks.slice(0)
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
        var primary = primaryDisk();
        var rows = metrics && metrics.disk && metrics.disk.disks
            ? metrics.disk.disks.slice(0)
            : [];
        rows = rows.filter(function(item) {
            return !primary || item.mount_point !== primary.mount_point;
        });
        rows.sort(function(a, b) { return (b.usage_percent || 0) - (a.usage_percent || 0); });
        return rows.slice(0, 4);
    }

    Layout.fillWidth: true
    spacing: theme.spacingM

    // ── Hero: disco principal ───────────────────────────────────────────────
    MetricCard {
        Layout.fillWidth: true
        hero: true
        title: "Disk"
        subtitle: primaryDisk()
            ? primaryDisk().name + " · " + primaryDisk().mount_point
            : "Sem dados"

        HeroMetric {
            Layout.fillWidth: true
            label: "Uso principal"
            value: primaryDisk() ? String(Math.round(primaryDisk().usage_percent)) : "0"
            unit: "%"
            accentColor: theme.diskColor
            footnote: primaryDisk()
                ? (root.fmtOne(primaryDisk().used_space) + " de " + root.fmtOne(primaryDisk().total_space) + " GB")
                : "-"
        }

        MetricBar {
            visible: primaryDisk() !== null
            label: primaryDisk() ? primaryDisk().mount_point : "-"
            value: primaryDisk() ? primaryDisk().usage_percent : 0
            barColor: theme.diskColor
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.successColor
            label: "Livre"
            value: primaryDisk() ? root.fmtOne(primaryDisk().available_space) + " GB" : "-"
        }
    }

    // ── I/O Activity ────────────────────────────────────────────────────────
    MetricCard {
        Layout.fillWidth: true
        title: "I/O Activity"
        subtitle: "Leitura e escrita agregadas"

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingM

            HeroMetric {
                Layout.fillWidth: true
                label: "Leitura"
                value: root.fmtRate(root.diskReadRate)
                accentColor: theme.diskColor
            }

            HeroMetric {
                Layout.fillWidth: true
                label: "Escrita"
                value: root.fmtRate(root.diskWriteRate)
                accentColor: theme.dangerColor
            }
        }

        SectionHeader {
            title: "Read"
            subtitle: root.fmtRate(root.diskReadRate)
        }

        HistoryChart {
            Layout.fillWidth: true
            values: root.diskReadHistory
            strokeColor: theme.diskColor
            fillColor: Qt.rgba(0.204, 0.827, 0.600, 0.18)
            maximumValue: root.ioHistoryMaximum()
            maxLabel: root.fmtRate(root.ioHistoryMaximum())
            minLabel: "0 B/s"
        }

        SectionHeader {
            title: "Write"
            subtitle: root.fmtRate(root.diskWriteRate)
        }

        HistoryChart {
            Layout.fillWidth: true
            values: root.diskWriteHistory
            strokeColor: theme.dangerColor
            fillColor: Qt.rgba(0.937, 0.267, 0.267, 0.18)
            maximumValue: root.ioHistoryMaximum()
            maxLabel: root.fmtRate(root.ioHistoryMaximum())
            minLabel: "0 B/s"
        }
    }

    // ── Partições secundárias ───────────────────────────────────────────────
    MetricCard {
        Layout.fillWidth: true
        title: "Partições"
        subtitle: "Secundárias por uso"

        Repeater {
            model: root.secondaryDisks()

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
                    label: root.fmtOne(modelData.used_space) + " de " + root.fmtOne(modelData.total_space) + " GB"
                    value: root.fmtOne(modelData.available_space) + " GB livres"
                }
            }
        }
    }

    Theme { id: theme }
}
