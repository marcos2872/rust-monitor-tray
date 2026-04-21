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

    function fmtPercent(value) {
        if (value === undefined || value === null || isNaN(value)) return "0%";
        return Math.round(Number(value)) + "%";
    }

    function historyWindowLabel() {
        return "Últimos " + Math.max(1, Math.round(historyDurationMs / 60000)) + " min";
    }

    Layout.fillWidth: true
    spacing: theme.spacingM

    MetricCard {
        Layout.fillWidth: true
        title: "CPU"
        subtitle: metrics && metrics.cpu ? metrics.cpu.name : "Sem dados"

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingS

            StatusChip {
                text: metrics && metrics.cpu && metrics.cpu.usage_percent >= 80 ? "Alto" : (metrics && metrics.cpu && metrics.cpu.usage_percent >= 50 ? "Médio" : "OK")
                chipColor: metrics && metrics.cpu && metrics.cpu.usage_percent >= 80 ? theme.dangerColor : (metrics && metrics.cpu && metrics.cpu.usage_percent >= 50 ? theme.warningColor : theme.successColor)
            }

            MetricRow {
                Layout.fillWidth: true
                label: "Freq"
                value: metrics && metrics.cpu ? metrics.cpu.frequency + " MHz" : "-"
            }

            MetricRow {
                Layout.fillWidth: true
                label: "Núcleos"
                value: metrics && metrics.cpu ? metrics.cpu.core_count : "-"
            }
        }

        MetricBar {
            label: "Uso total"
            value: metrics && metrics.cpu ? metrics.cpu.usage_percent : 0
            barColor: theme.cpuColor
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Histórico"
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
        title: "Uso por núcleo"
        subtitle: "Resumo dos núcleos lógicos"

        GridLayout {
            Layout.fillWidth: true
            columns: 2
            columnSpacing: theme.spacingS
            rowSpacing: theme.spacingXS

            Repeater {
                model: metrics && metrics.cpu && metrics.cpu.per_core_usage ? metrics.cpu.per_core_usage : []

                delegate: MetricRow {
                    Layout.fillWidth: true
                    label: "Core " + String(index + 1).padStart(2, "0")
                    value: root.fmtPercent(modelData)
                }
            }
        }
    }

    Theme { id: theme }
}
