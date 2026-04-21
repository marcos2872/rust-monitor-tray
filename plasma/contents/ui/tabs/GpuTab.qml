import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import "../components"
import ".."

ColumnLayout {
    id: root

    property var metrics: ({})
    property var gpuHistory: []
    property int historyDurationMs: 5 * 60 * 1000

    function historyWindowLabel() {
        return "Últimos " + Math.max(1, Math.round(historyDurationMs / 60000)) + " min";
    }

    function vendorLabel(vendor) {
        if (vendor === "amd")    return "AMD";
        if (vendor === "nvidia") return "NVIDIA";
        if (vendor === "intel")  return "Intel";
        return "GPU";
    }

    function fmtPercent(value) {
        if (value === null || value === undefined || isNaN(value)) return "0%";
        return Math.round(Number(value)) + "%";
    }

    function temperatureAccentColor(celsius) {
        var t = celsius || 0;
        if (t >= 85) return theme.dangerColor;
        if (t >= 70) return theme.warningColor;
        return theme.successColor;
    }

    Layout.fillWidth: true
    spacing: theme.spacingM

    // ── Sem GPU ─────────────────────────────────────────────────────────────
    MetricCard {
        visible: !metrics.gpus || metrics.gpus.length === 0
        Layout.fillWidth: true
        title: "GPU"
        subtitle: "Nenhuma GPU detectada"

        PlasmaComponents3.Label {
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
            color: theme.mutedTextColor
            text: "Nenhuma GPU compatível foi encontrada.\n" +
                  "AMD e Intel são detectadas via /sys/class/drm.\n" +
                  "NVIDIA requer nvidia-smi instalado."
        }
    }

    // ── Uma entrada por GPU detectada ────────────────────────────────────────
    Repeater {
        model: metrics.gpus || []

        delegate: ColumnLayout {
            Layout.fillWidth: true
            spacing: theme.spacingM

            // Hero: temperatura · uso · potência
            MetricCard {
                Layout.fillWidth: true
                hero: true
                title: modelData.name || "GPU"
                subtitle: root.vendorLabel(modelData.vendor)

                RowLayout {
                    Layout.fillWidth: true
                    spacing: theme.spacingM

                    HeroMetric {
                        Layout.preferredWidth: 110
                        label: "Temperatura"
                        value: modelData.temperature_celsius !== null && modelData.temperature_celsius !== undefined
                            ? Number(modelData.temperature_celsius).toFixed(1)
                            : "-"
                        unit: modelData.temperature_celsius !== null && modelData.temperature_celsius !== undefined
                            ? "°C" : ""
                        accentColor: root.temperatureAccentColor(modelData.temperature_celsius)
                        footnote: modelData.fan_rpm !== null && modelData.fan_rpm !== undefined
                            ? modelData.fan_rpm + " RPM"
                            : "sem sensor"
                    }

                    RingGauge {
                        Layout.alignment: Qt.AlignHCenter
                        value: modelData.usage_percent || 0
                        centerText: root.fmtPercent(modelData.usage_percent)
                        label: modelData.usage_percent !== null ? "Uso GPU" : "Sem dados"
                        accentColor: theme.gpuColor
                    }

                    HeroMetric {
                        Layout.preferredWidth: 110
                        label: "Potência"
                        value: modelData.power_watts !== null && modelData.power_watts !== undefined
                            ? Number(modelData.power_watts).toFixed(1)
                            : "-"
                        unit: modelData.power_watts !== null && modelData.power_watts !== undefined
                            ? "W" : ""
                        accentColor: theme.warningColor
                        footnote: modelData.vendor !== "intel" ? "consumo atual" : "não disponível"
                    }
                }
            }

            // Histórico de uso (apenas para GPU primária)
            MetricCard {
                visible: index === 0 && modelData.usage_percent !== null && modelData.usage_percent !== undefined
                Layout.fillWidth: true
                title: "Usage history"
                subtitle: root.historyWindowLabel()

                HistoryChart {
                    Layout.fillWidth: true
                    values: root.gpuHistory
                    strokeColor: theme.gpuColor
                    maximumValue: 100
                    maxLabel: "100%"
                    minLabel: "0%"
                }
            }

            // VRAM
            MetricCard {
                visible: modelData.vram_total_gb !== null && modelData.vram_total_gb !== undefined
                Layout.fillWidth: true
                title: "VRAM"
                subtitle: modelData.vram_total_gb !== null
                    ? theme.fmtOne(modelData.vram_used_gb) + " / " + theme.fmtOne(modelData.vram_total_gb) + " GB"
                    : "Não disponível (UMA)"

                MetricBar {
                    Layout.fillWidth: true
                    label: "Uso"
                    value: modelData.vram_usage_percent || 0
                    barColor: theme.memoryColor
                }

                MetricRow {
                    Layout.fillWidth: true
                    accentColor: theme.memoryColor
                    label: "Usada"
                    value: theme.fmtOne(modelData.vram_used_gb) + " GB"
                }

                MetricRow {
                    Layout.fillWidth: true
                    accentColor: theme.systemColor
                    label: "Total"
                    value: theme.fmtOne(modelData.vram_total_gb) + " GB"
                }
            }

            // Details: clocks e informações extras
            MetricCard {
                Layout.fillWidth: true
                title: "Details"
                subtitle: root.vendorLabel(modelData.vendor) + " · " + modelData.name

                MetricRow {
                    visible: modelData.shader_clock_mhz !== null && modelData.shader_clock_mhz !== undefined
                    Layout.fillWidth: true
                    accentColor: theme.gpuColor
                    label: "Shader clock"
                    value: (modelData.shader_clock_mhz || 0) + " MHz"
                }

                MetricRow {
                    visible: modelData.memory_clock_mhz !== null && modelData.memory_clock_mhz !== undefined
                    Layout.fillWidth: true
                    accentColor: theme.memoryColor
                    label: "Mem clock"
                    value: (modelData.memory_clock_mhz || 0) + " MHz"
                }

                MetricRow {
                    visible: modelData.fan_rpm !== null && modelData.fan_rpm !== undefined
                    Layout.fillWidth: true
                    accentColor: theme.systemColor
                    label: "Fan"
                    value: (modelData.fan_rpm || 0) + " RPM"
                }

                MetricRow {
                    Layout.fillWidth: true
                    accentColor: "transparent"
                    label: "Driver"
                    value: root.vendorLabel(modelData.vendor)
                }
            }
        }
    }

    Theme { id: theme }
}
