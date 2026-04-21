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

    Layout.fillWidth: true
    spacing: theme.spacingM

    MetricCard {
        Layout.fillWidth: true
        title: "Memória"
        subtitle: metrics && metrics.memory ? (root.fmtOne(metrics.memory.used_memory) + " / " + root.fmtOne(metrics.memory.total_memory) + " GB") : "Sem dados"

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingS

            StatusChip {
                text: metrics && metrics.memory && metrics.memory.usage_percent >= 80 ? "Alto" : (metrics && metrics.memory && metrics.memory.usage_percent >= 50 ? "Médio" : "OK")
                chipColor: metrics && metrics.memory && metrics.memory.usage_percent >= 80 ? theme.dangerColor : (metrics && metrics.memory && metrics.memory.usage_percent >= 50 ? theme.warningColor : theme.successColor)
            }

            MetricRow {
                Layout.fillWidth: true
                label: "Livre"
                value: metrics && metrics.memory ? root.fmtOne(metrics.memory.available_memory) + " GB" : "-"
            }
        }

        MetricBar {
            label: "RAM"
            value: metrics && metrics.memory ? metrics.memory.usage_percent : 0
            barColor: theme.memoryColor
        }

        GridLayout {
            Layout.fillWidth: true
            columns: 2
            columnSpacing: theme.spacingS
            rowSpacing: theme.spacingXS

            MetricRow {
                Layout.fillWidth: true
                label: "Usada"
                value: metrics && metrics.memory ? root.fmtOne(metrics.memory.used_memory) + " GB" : "-"
            }

            MetricRow {
                Layout.fillWidth: true
                label: "Total"
                value: metrics && metrics.memory ? root.fmtOne(metrics.memory.total_memory) + " GB" : "-"
            }
        }

        MetricBar {
            visible: metrics && metrics.memory && metrics.memory.total_swap > 0
            label: "Swap"
            value: metrics && metrics.memory && metrics.memory.total_swap > 0 ? (metrics.memory.used_swap / metrics.memory.total_swap) * 100.0 : 0
            barColor: "#a78bfa"
        }
    }

    Theme { id: theme }
}
