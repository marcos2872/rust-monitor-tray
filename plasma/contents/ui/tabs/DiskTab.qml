import QtQuick 2.15
import QtQuick.Layouts 1.15
import "../components"
import ".."

ColumnLayout {
    id: root

    property var metrics: ({})

    function fmtOne(value) {
        if (value === undefined || value === null || isNaN(value)) return "0.0";
        return Number(value).toFixed(1);
    }

    function primaryDisk() {
        var rows = metrics && metrics.disk && metrics.disk.disks ? metrics.disk.disks.slice(0) : [];
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
        var rows = metrics && metrics.disk && metrics.disk.disks ? metrics.disk.disks.slice(0) : [];
        rows = rows.filter(function(item) {
            return !primary || item.mount_point !== primary.mount_point;
        });
        rows.sort(function(a, b) { return (b.usage_percent || 0) - (a.usage_percent || 0); });
        return rows.slice(0, 4);
    }

    Layout.fillWidth: true
    spacing: theme.spacingM

    MetricCard {
        Layout.fillWidth: true
        hero: true
        title: "Disk"
        subtitle: primaryDisk() ? primaryDisk().name + " · " + primaryDisk().mount_point : "Sem dados"

        HeroMetric {
            Layout.fillWidth: true
            label: "Uso principal"
            value: primaryDisk() ? String(Math.round(primaryDisk().usage_percent)) : "0"
            unit: "%"
            accentColor: theme.diskColor
            footnote: primaryDisk() ? (root.fmtOne(primaryDisk().used_space) + " de " + root.fmtOne(primaryDisk().total_space) + " GB") : "-"
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

    MetricCard {
        Layout.fillWidth: true
        title: "Partições"
        subtitle: "Secundárias por uso"

        Repeater {
            model: root.secondaryDisks()

            delegate: ColumnLayout {
                Layout.fillWidth: true
                spacing: theme.spacingXS

                MetricRow {
                    Layout.fillWidth: true
                    accentColor: theme.diskColor
                    label: modelData.mount_point
                    value: Math.round(modelData.usage_percent) + "%"
                }

                MetricBar {
                    Layout.fillWidth: true
                    label: modelData.name
                    value: modelData.usage_percent
                    barColor: theme.diskColor
                    barHeight: 8
                }
            }
        }
    }

    Theme { id: theme }
}
