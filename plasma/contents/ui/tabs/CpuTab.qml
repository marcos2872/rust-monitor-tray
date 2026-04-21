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

    function fmtUptime(seconds) {
        if (seconds === undefined || seconds === null || isNaN(seconds)) return "0m";
        var total = Number(seconds);
        var days    = Math.floor(total / 86400);
        var hours   = Math.floor((total % 86400) / 3600);
        var minutes = Math.floor((total % 3600) / 60);
        if (days > 0)  return days + "d " + hours + "h " + minutes + "m";
        if (hours > 0) return hours + "h " + minutes + "m";
        return minutes + "m";
    }

    function historyWindowLabel() {
        return "Últimos " + Math.max(1, Math.round(historyDurationMs / 60000)) + " min";
    }

    function hottestTemp() {
        return metrics && metrics.sensors
            ? metrics.sensors.hottest_temperature_celsius
            : null;
    }

    Layout.fillWidth: true
    spacing: theme.spacingM

    // ── Hero: temperatura · uso total · load 1 min ──────────────────────────
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
                label: "Temperatura"
                value: root.hottestTemp() !== null && root.hottestTemp() !== undefined
                    ? Number(root.hottestTemp()).toFixed(1)
                    : "-"
                unit: root.hottestTemp() !== null && root.hottestTemp() !== undefined ? "°C" : ""
                accentColor: theme.dangerColor
                footnote: metrics && metrics.sensors && metrics.sensors.hottest_label
                    ? metrics.sensors.hottest_label
                    : "sem sensor"
            }

            RingGauge {
                Layout.alignment: Qt.AlignHCenter
                value: metrics && metrics.cpu ? metrics.cpu.usage_percent : 0
                centerText: root.fmtPercent(metrics && metrics.cpu ? metrics.cpu.usage_percent : 0)
                label: "Uso total"
                accentColor: theme.cpuColor
            }

            HeroMetric {
                Layout.preferredWidth: 110
                label: "Carga 1m"
                value: metrics && metrics.load_average
                    ? Number(metrics.load_average[0]).toFixed(2)
                    : "0.00"
                accentColor: theme.warningColor
                footnote: "load average"
            }
        }
    }

    // ── Histórico de uso ────────────────────────────────────────────────────
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

    // ── Details: user / system / idle / uptime ──────────────────────────────
    MetricCard {
        Layout.fillWidth: true
        title: "Details"
        subtitle: "Distribuição do uso"

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.cpuColor
            label: "User"
            value: root.fmtPercent(metrics && metrics.cpu ? metrics.cpu.user_percent : 0)
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.dangerColor
            label: "System"
            value: root.fmtPercent(metrics && metrics.cpu ? metrics.cpu.system_percent : 0)
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.successColor
            label: "Idle"
            value: root.fmtPercent(metrics && metrics.cpu ? metrics.cpu.idle_percent : 0)
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.systemColor
            label: "Uptime"
            value: root.fmtUptime(metrics ? metrics.uptime : 0)
        }
    }

    // ── Grade de uso por núcleo ─────────────────────────────────────────────
    MetricCard {
        Layout.fillWidth: true
        title: "Core usage"
        subtitle: metrics && metrics.cpu
            ? metrics.cpu.core_count + " núcleos"
            : "-"

        GridLayout {
            Layout.fillWidth: true
            columns: 2
            columnSpacing: theme.spacingM
            rowSpacing: theme.spacingXS

            Repeater {
                model: metrics && metrics.cpu && metrics.cpu.per_core_usage
                    ? metrics.cpu.per_core_usage
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

    // ── Average load: 1 / 5 / 15 minutos ───────────────────────────────────
    MetricCard {
        Layout.fillWidth: true
        title: "Average load"
        subtitle: "Média de carga do sistema"

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.warningColor
            label: "1 minuto"
            value: metrics && metrics.load_average
                ? Number(metrics.load_average[0]).toFixed(2)
                : "0.00"
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.warningColor
            label: "5 minutos"
            value: metrics && metrics.load_average
                ? Number(metrics.load_average[1]).toFixed(2)
                : "0.00"
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.warningColor
            label: "15 minutos"
            value: metrics && metrics.load_average
                ? Number(metrics.load_average[2]).toFixed(2)
                : "0.00"
        }
    }

    // ── Frequência ──────────────────────────────────────────────────────────
    MetricCard {
        Layout.fillWidth: true
        title: "Frequency"
        subtitle: "Frequência dos núcleos"

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.cpuColor
            label: "Todos os núcleos"
            value: metrics && metrics.cpu
                ? metrics.cpu.frequency + " MHz"
                : "-"
        }
    }

    Theme { id: theme }
}
