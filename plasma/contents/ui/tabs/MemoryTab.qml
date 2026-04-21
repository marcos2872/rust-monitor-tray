import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import "../components"
import ".."

ColumnLayout {
    id: root

    property var metrics: ({})
    property var history: []
    property int historyDurationMs: 5 * 60 * 1000

    function fmtOne(value) {
        if (value === undefined || value === null || isNaN(value)) return "0.0";
        return Number(value).toFixed(1);
    }

    function historyWindowLabel() {
        return "Últimos " + Math.max(1, Math.round(historyDurationMs / 60000)) + " min";
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

    MetricCard {
        Layout.fillWidth: true
        title: "Histórico"
        subtitle: root.historyWindowLabel()

        HistoryChart {
            Layout.fillWidth: true
            values: root.history
            strokeColor: theme.memoryColor
            fillColor: Qt.rgba(0.753, 0.518, 0.988, 0.18)
            maximumValue: 100
            maxLabel: "100%"
            minLabel: "0%"
        }
    }

    Theme { id: theme }
}
