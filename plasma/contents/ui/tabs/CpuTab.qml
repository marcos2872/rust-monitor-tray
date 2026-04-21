import QtQuick 2.15
import QtQuick.Layouts 1.15
import "../components"
import ".."

ColumnLayout {
    id: root

    property var cpuMetrics: ({})
    property var sensorMetrics: ({})
    property int uptime: 0
    property var loadAverage: [0, 0, 0]
    property var history: ({})
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
        hero: true
        title: "CPU"
        subtitle: root.cpuMetrics ? root.cpuMetrics.name : "Sem dados"

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingM

            HeroMetric {
                Layout.preferredWidth: 110
                label: "Temperatura"
                value: root.sensorMetrics && root.sensorMetrics.hottest_cpu_celsius !== null
                       && root.sensorMetrics.hottest_cpu_celsius !== undefined
                    ? Number(root.sensorMetrics.hottest_cpu_celsius).toFixed(1) : "-"
                unit: root.sensorMetrics && root.sensorMetrics.hottest_cpu_celsius !== null
                      && root.sensorMetrics.hottest_cpu_celsius !== undefined ? "°C" : ""
                accentColor: theme.dangerColor
                footnote: root.sensorMetrics && root.sensorMetrics.hottest_cpu_label
                    ? root.sensorMetrics.hottest_cpu_label : "sem sensor de CPU"
            }

            RingGauge {
                Layout.alignment: Qt.AlignHCenter
                value: root.cpuMetrics ? root.cpuMetrics.usage_percent : 0
                centerText: root.fmtPercent(root.cpuMetrics ? root.cpuMetrics.usage_percent : 0)
                label: "Uso total"
                accentColor: theme.cpuColor
            }

            HeroMetric {
                Layout.preferredWidth: 110
                label: "Carga 1m"
                value: root.loadAverage ? Number(root.loadAverage[0]).toFixed(2) : "0.00"
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
            series: root.history
            strokeColor: theme.cpuColor
            maximumValue: 100
            maxLabel: "100%"
            minLabel: "0%"
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Details"
        subtitle: "Distribuição do uso"

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.cpuColor
            label: "User"
            value: root.fmtPercent(root.cpuMetrics ? root.cpuMetrics.user_percent : 0)
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.dangerColor
            label: "System"
            value: root.fmtPercent(root.cpuMetrics ? root.cpuMetrics.system_percent : 0)
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.successColor
            label: "Idle"
            value: root.fmtPercent(root.cpuMetrics ? root.cpuMetrics.idle_percent : 0)
        }

        MetricRow {
            visible: root.cpuMetrics && root.cpuMetrics.steal_percent > 0.1
            Layout.fillWidth: true
            accentColor: theme.dangerColor
            label: "Steal"
            value: root.fmtPercent(root.cpuMetrics ? root.cpuMetrics.steal_percent : 0)
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.systemColor
            label: "Uptime"
            value: theme.fmtUptime(root.uptime)
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Core usage"
        subtitle: root.cpuMetrics ? root.cpuMetrics.core_count + " núcleos" : "-"

        GridLayout {
            Layout.fillWidth: true
            columns: 2
            columnSpacing: theme.spacingM
            rowSpacing: theme.spacingXS

            Repeater {
                model: root.cpuMetrics && root.cpuMetrics.per_core_usage
                    ? root.cpuMetrics.per_core_usage
                    : []

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

    MetricCard {
        Layout.fillWidth: true
        title: "Average load"
        subtitle: "Média de carga do sistema"

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.warningColor
            label: "1 minuto"
            value: root.loadAverage ? Number(root.loadAverage[0]).toFixed(2) : "0.00"
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.warningColor
            label: "5 minutos"
            value: root.loadAverage ? Number(root.loadAverage[1]).toFixed(2) : "0.00"
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.warningColor
            label: "15 minutos"
            value: root.loadAverage ? Number(root.loadAverage[2]).toFixed(2) : "0.00"
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Frequency"
        subtitle: "Frequência dos núcleos"

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.cpuColor
            label: "Todos os núcleos"
            value: root.cpuMetrics ? root.cpuMetrics.frequency + " MHz" : "-"
        }
    }

    Theme { id: theme }
}
