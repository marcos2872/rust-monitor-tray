import QtQuick 2.15
import QtQuick.Layouts 1.15
import "../components"
import ".."

ColumnLayout {
    id: root

    property var metrics: ({})
    property var history: []
    property int historyDurationMs: 5 * 60 * 1000

    function historyWindowLabel() {
        return "Últimos " + Math.max(1, Math.round(historyDurationMs / 60000)) + " min";
    }

    function swapPercent() {
        if (!metrics || !metrics.memory || !metrics.memory.total_swap) return 0;
        return (metrics.memory.used_swap / metrics.memory.total_swap) * 100.0;
    }

    Layout.fillWidth: true
    spacing: theme.spacingM

    MetricCard {
        Layout.fillWidth: true
        hero: true
        title: "RAM"
        subtitle: metrics && metrics.memory ? (theme.fmtOne(metrics.memory.used_memory) + " / " + theme.fmtOne(metrics.memory.total_memory) + " GB") : "Sem dados"

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingM

            HeroMetric {
                Layout.preferredWidth: 110
                label: "Livre"
                value: metrics && metrics.memory ? theme.fmtOne(metrics.memory.available_memory) : "0.0"
                unit: "GB"
                accentColor: theme.successColor
                footnote: "disponível"
            }

            RingGauge {
                Layout.alignment: Qt.AlignHCenter
                value: metrics && metrics.memory ? metrics.memory.usage_percent : 0
                centerText: Math.round(metrics && metrics.memory ? metrics.memory.usage_percent : 0) + "%"
                label: "Memória"
                accentColor: theme.memoryColor
            }

            HeroMetric {
                Layout.preferredWidth: 110
                label: "Swap"
                value: metrics && metrics.memory ? theme.fmtOne(metrics.memory.used_swap) : "0.0"
                unit: "GB"
                accentColor: theme.swapColor
                footnote: metrics && metrics.memory && metrics.memory.total_swap > 0 ? Math.round(root.swapPercent()) + "% usado" : "sem swap"
            }
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Usage history"
        subtitle: root.historyWindowLabel()

        HistoryChart {
            Layout.fillWidth: true
            values: root.history
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
            value: metrics && metrics.memory ? theme.fmtOne(metrics.memory.used_memory) + " GB" : "-"
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.successColor
            label: "Livre"
            value: metrics && metrics.memory ? theme.fmtOne(metrics.memory.available_memory) + " GB" : "-"
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.swapColor
            label: "Swap"
            value: metrics && metrics.memory ? theme.fmtOne(metrics.memory.used_swap) + " / " + theme.fmtOne(metrics.memory.total_swap) + " GB" : "-"
        }
    }

    Theme { id: theme }
}
