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

    function hottestTemp() {
        // Filtra apenas sensores de CPU (coretemp, k10temp, zenpower)
        var sensors = metrics && metrics.sensors && metrics.sensors.temperatures
            ? metrics.sensors.temperatures : [];
        var cpuSensors = sensors.filter(function(s) {
            var chipName = (s.chip || "").toLowerCase();
            return chipName === "coretemp" || chipName === "k10temp" || chipName === "zenpower"
                || chipName.indexOf("cpu") >= 0;
        });
        if (cpuSensors.length === 0) return null;
        var max = cpuSensors[0].temperature_celsius;
        for (var i = 1; i < cpuSensors.length; i++) {
            if (cpuSensors[i].temperature_celsius > max)
                max = cpuSensors[i].temperature_celsius;
        }
        return max;
    }

    function hottestCpuLabel() {
        var sensors = metrics && metrics.sensors && metrics.sensors.temperatures
            ? metrics.sensors.temperatures : [];
        var cpuSensors = sensors.filter(function(s) {
            var chipName = (s.chip || "").toLowerCase();
            return chipName === "coretemp" || chipName === "k10temp" || chipName === "zenpower"
                || chipName.indexOf("cpu") >= 0;
        });
        if (cpuSensors.length === 0) return "sem sensor de CPU";
        var best = cpuSensors[0];
        for (var i = 1; i < cpuSensors.length; i++) {
            if (cpuSensors[i].temperature_celsius > best.temperature_celsius)
                best = cpuSensors[i];
        }
        return best.label;
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
                value: root.hottestTemp() !== null
                    ? Number(root.hottestTemp()).toFixed(1)
                    : "-"
                unit: root.hottestTemp() !== null ? "°C" : ""
                accentColor: theme.dangerColor
                footnote: root.hottestCpuLabel()
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
            visible: metrics && metrics.cpu && metrics.cpu.steal_percent > 0.1
            Layout.fillWidth: true
            accentColor: theme.dangerColor
            label: "Steal"
            value: root.fmtPercent(metrics && metrics.cpu ? metrics.cpu.steal_percent : 0)
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.systemColor
            label: "Uptime"
            value: theme.fmtUptime(metrics ? metrics.uptime : 0)
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
