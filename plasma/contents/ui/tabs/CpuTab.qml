import QtQuick 2.15
import QtQuick.Layouts 1.15
import "../components"
import ".."

ColumnLayout {
    id: root

    property var metrics: ({})
    property var history: []
    property int historyDurationMs: 5 * 60 * 1000

    function fmtPercent(value) {
        if (value === undefined || value === null || isNaN(value)) return "0%";
        return Math.round(Number(value)) + "%";
    }

    function historyWindowLabel() {
        return "Últimos " + Math.max(1, Math.round(historyDurationMs / 60000)) + " min";
    }

    function loadOneMinute() {
        return metrics && metrics.load_average ? Number(metrics.load_average[0]).toFixed(2) : "0.00";
    }

    Layout.fillWidth: true
    spacing: theme.spacingM

    MetricCard {
        Layout.fillWidth: true
        hero: true
        title: "CPU"
        subtitle: metrics && metrics.cpu ? metrics.cpu.name : "Sem dados"

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingM

            HeroMetric {
                Layout.preferredWidth: 110
                label: "Frequência"
                value: metrics && metrics.cpu ? String(metrics.cpu.frequency) : "0"
                unit: "MHz"
                accentColor: theme.cpuColor
                footnote: metrics && metrics.cpu ? metrics.cpu.core_count + " núcleos" : "-"
            }

            RingGauge {
                Layout.alignment: Qt.AlignHCenter
                value: metrics && metrics.cpu ? metrics.cpu.usage_percent : 0
                centerText: root.fmtPercent(metrics && metrics.cpu ? metrics.cpu.usage_percent : 0)
                label: "Uso total"
                accentColor: theme.cpuColor
                footnote: root.historyWindowLabel()
            }

            HeroMetric {
                Layout.preferredWidth: 110
                label: "Carga 1m"
                value: root.loadOneMinute()
                accentColor: theme.warningColor
                footnote: "load average"
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
            strokeColor: theme.cpuColor
            fillColor: Qt.rgba(0.376, 0.647, 0.98, 0.18)
            maximumValue: 100
            maxLabel: "100%"
            minLabel: "0%"
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Details"
        subtitle: "Uso por núcleo"

        GridLayout {
            Layout.fillWidth: true
            columns: 2
            columnSpacing: theme.spacingM
            rowSpacing: theme.spacingXS

            Repeater {
                model: metrics && metrics.cpu && metrics.cpu.per_core_usage ? metrics.cpu.per_core_usage : []

                delegate: MetricRow {
                    Layout.fillWidth: true
                    dense: true
                    accentColor: theme.cpuColor
                    label: "Core " + String(index + 1).padStart(2, "0")
                    value: root.fmtPercent(modelData)
                }
            }
        }
    }

    Theme { id: theme }
}
