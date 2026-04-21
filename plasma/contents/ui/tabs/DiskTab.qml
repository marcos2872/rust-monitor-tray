import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import "../components"
import ".."

ColumnLayout {
    id: root

    property var metrics: ({})

    function fmtOne(value) {
        if (value === undefined || value === null || isNaN(value)) return "0.0";
        return Number(value).toFixed(1);
    }

    function usage() {
        if (!metrics || !metrics.disk || !metrics.disk.total_space) return 0;
        return (metrics.disk.used_space / metrics.disk.total_space) * 100.0;
    }

    function topDisks() {
        var rows = metrics && metrics.disk && metrics.disk.disks ? metrics.disk.disks.slice(0) : [];
        rows.sort(function(a, b) { return (b.usage_percent || 0) - (a.usage_percent || 0); });
        return rows.slice(0, 5);
    }

    Layout.fillWidth: true
    spacing: theme.spacingM

    MetricCard {
        Layout.fillWidth: true
        title: "Disco"
        subtitle: metrics && metrics.disk ? (root.fmtOne(metrics.disk.used_space) + " / " + root.fmtOne(metrics.disk.total_space) + " GB") : "Sem dados"

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingS

            StatusChip {
                text: root.usage() >= 80 ? "Alto" : (root.usage() >= 50 ? "Médio" : "OK")
                chipColor: root.usage() >= 80 ? theme.dangerColor : (root.usage() >= 50 ? theme.warningColor : theme.successColor)
            }

            MetricRow {
                Layout.fillWidth: true
                label: "Livre"
                value: metrics && metrics.disk ? root.fmtOne(metrics.disk.available_space) + " GB" : "-"
            }
        }

        MetricBar {
            label: "Uso agregado"
            value: root.usage()
            barColor: theme.diskColor
        }

        SectionHeader {
            title: "Partições principais"
            subtitle: "Ordenadas por uso"
        }

        Repeater {
            model: root.topDisks()

            delegate: MetricRow {
                Layout.fillWidth: true
                label: modelData.mount_point
                value: Math.round(modelData.usage_percent) + "%"
            }
        }
    }

    Theme { id: theme }
}
