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
        var primary = root.cachedPrimaryDisk;
        var rows = metrics && metrics.disk && metrics.disk.disks
            ? metrics.disk.disks.slice(0)
            : [];
        rows = rows.filter(function(item) {
            return !primary || item.mount_point !== primary.mount_point;
        });
        rows.sort(function(a, b) { return (b.usage_percent || 0) - (a.usage_percent || 0); });
        return rows.slice(0, 4);
    }

    // Propriedades calculadas uma vez por ciclo de métricas
    readonly property var cachedPrimaryDisk: metrics ? primaryDisk() : null
    readonly property var cachedSecondaryDisks: metrics ? secondaryDisks() : []

    Layout.fillWidth: true
    spacing: theme.spacingM

    // ── Hero: disco principal ───────────────────────────────────────────────
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
            values: root.diskReadHistory
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
            values: root.diskWriteHistory
            strokeColor: theme.dangerColor
            maximumValue: root.ioHistoryMaximum()
            maxLabel: theme.fmtRate(root.ioHistoryMaximum())
            minLabel: "0 B/s"
        }
    }

    // ── Partições secundárias ───────────────────────────────────────────────
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
