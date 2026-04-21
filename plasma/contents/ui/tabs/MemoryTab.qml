import QtQuick 2.15
import QtQuick.Layouts 1.15
import "../components"
import ".."

ColumnLayout {
    id: root

    property var memoryMetrics: ({})
    property var history: ({})
    property int historyDurationMs: 5 * 60 * 1000

    function historyWindowLabel() {
        return "Últimos " + Math.max(1, Math.round(historyDurationMs / 60000)) + " min";
    }

    function swapPercent() {
        if (!root.memoryMetrics || !root.memoryMetrics.total_swap) return 0;
        return (root.memoryMetrics.used_swap / root.memoryMetrics.total_swap) * 100.0;
    }

    Layout.fillWidth: true
    spacing: theme.spacingM

    MetricCard {
        Layout.fillWidth: true
        hero: true
        title: "RAM"
        subtitle: root.memoryMetrics ? (theme.fmtOne(root.memoryMetrics.used_memory) + " / " + theme.fmtOne(root.memoryMetrics.total_memory) + " GB") : "Sem dados"

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingM

            HeroMetric {
                Layout.preferredWidth: 110
                label: "Livre"
                value: root.memoryMetrics ? theme.fmtOne(root.memoryMetrics.available_memory) : "0.0"
                unit: "GB"
                accentColor: theme.successColor
                footnote: "disponível"
            }

            RingGauge {
                Layout.alignment: Qt.AlignHCenter
                value: root.memoryMetrics ? root.memoryMetrics.usage_percent : 0
                centerText: Math.round(root.memoryMetrics ? root.memoryMetrics.usage_percent : 0) + "%"
                label: "Memória"
                accentColor: theme.memoryColor
            }

            HeroMetric {
                Layout.preferredWidth: 110
                label: "Swap"
                value: root.memoryMetrics ? theme.fmtOne(root.memoryMetrics.used_swap) : "0.0"
                unit: "GB"
                accentColor: theme.swapColor
                footnote: root.memoryMetrics && root.memoryMetrics.total_swap > 0 ? Math.round(root.swapPercent()) + "% usado" : "sem swap"
            }
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Usage history"
        subtitle: root.historyWindowLabel()

        HistoryChart {
            Layout.fillWidth: true
            series: root.history
            strokeColor: theme.memoryColor
            maximumValue: 100
            maxLabel: "100%"
            minLabel: "0%"
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Details"
        subtitle: "Resumo da memória"

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.memoryColor
            label: "Usada"
            value: root.memoryMetrics ? theme.fmtOne(root.memoryMetrics.used_memory) + " GB" : "-"
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.successColor
            label: "Livre"
            value: root.memoryMetrics ? theme.fmtOne(root.memoryMetrics.available_memory) + " GB" : "-"
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.swapColor
            label: "Swap"
            value: root.memoryMetrics ? theme.fmtOne(root.memoryMetrics.used_swap) + " / " + theme.fmtOne(root.memoryMetrics.total_swap) + " GB" : "-"
        }
    }

    Theme { id: theme }
}
