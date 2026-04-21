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

    // Acesso direto sem Repeater — evita recriação de delegates a cada atualização
    readonly property var primaryGpu: metrics && metrics.gpus && metrics.gpus.length > 0
        ? metrics.gpus[0] : null
    readonly property var extraGpus: metrics && metrics.gpus && metrics.gpus.length > 1
        ? metrics.gpus.slice(1) : []

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
        visible: root.primaryGpu === null
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

    // ── GPU primária — binding direto, sem Repeater ──────────────────────────

    // Hero: temperatura · uso · potência
    MetricCard {
        visible: root.primaryGpu !== null
        Layout.fillWidth: true
        hero: true
        title: root.primaryGpu ? root.primaryGpu.name : "GPU"
        subtitle: root.primaryGpu ? root.vendorLabel(root.primaryGpu.vendor) : ""

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingM

            HeroMetric {
                Layout.preferredWidth: 110
                label: "Temperatura"
                value: root.primaryGpu && root.primaryGpu.temperature_celsius !== null
                    ? Number(root.primaryGpu.temperature_celsius).toFixed(1) : "-"
                unit: root.primaryGpu && root.primaryGpu.temperature_celsius !== null ? "°C" : ""
                accentColor: root.temperatureAccentColor(root.primaryGpu ? root.primaryGpu.temperature_celsius : 0)
                footnote: root.primaryGpu && root.primaryGpu.fan_rpm !== null
                    ? root.primaryGpu.fan_rpm + " RPM" : "sem sensor"
            }

            RingGauge {
                Layout.alignment: Qt.AlignHCenter
                value: root.primaryGpu ? (root.primaryGpu.usage_percent || 0) : 0
                centerText: root.fmtPercent(root.primaryGpu ? root.primaryGpu.usage_percent : null)
                label: root.primaryGpu && root.primaryGpu.usage_percent !== null ? "Uso GPU" : "Sem dados"
                accentColor: theme.gpuColor
            }

            HeroMetric {
                Layout.preferredWidth: 110
                label: "Potência"
                value: root.primaryGpu && root.primaryGpu.power_watts !== null
                    ? Number(root.primaryGpu.power_watts).toFixed(1) : "-"
                unit: root.primaryGpu && root.primaryGpu.power_watts !== null ? "W" : ""
                accentColor: theme.warningColor
                footnote: root.primaryGpu && root.primaryGpu.vendor !== "intel"
                    ? "consumo atual" : "não disponível"
            }
        }
    }

    // Histórico de uso
    MetricCard {
        visible: root.primaryGpu !== null && root.primaryGpu.usage_percent !== null
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
        visible: root.primaryGpu !== null && root.primaryGpu.vram_total_gb !== null
        Layout.fillWidth: true
        title: "VRAM"
        subtitle: root.primaryGpu && root.primaryGpu.vram_total_gb !== null
            ? theme.fmtOne(root.primaryGpu.vram_used_gb) + " / " + theme.fmtOne(root.primaryGpu.vram_total_gb) + " GB"
            : "Não disponível (UMA)"

        MetricBar {
            Layout.fillWidth: true
            label: "Uso"
            value: root.primaryGpu ? (root.primaryGpu.vram_usage_percent || 0) : 0
            barColor: theme.memoryColor
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.memoryColor
            label: "Usada"
            value: root.primaryGpu ? theme.fmtOne(root.primaryGpu.vram_used_gb) + " GB" : "-"
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.systemColor
            label: "Total"
            value: root.primaryGpu ? theme.fmtOne(root.primaryGpu.vram_total_gb) + " GB" : "-"
        }
    }

    // Details
    MetricCard {
        visible: root.primaryGpu !== null
        Layout.fillWidth: true
        title: "Details"
        subtitle: root.primaryGpu
            ? root.vendorLabel(root.primaryGpu.vendor) + " · " + root.primaryGpu.name
            : ""

        MetricRow {
            visible: root.primaryGpu && root.primaryGpu.shader_clock_mhz !== null
            Layout.fillWidth: true
            accentColor: theme.gpuColor
            label: "Shader clock"
            value: root.primaryGpu ? (root.primaryGpu.shader_clock_mhz || 0) + " MHz" : "-"
        }

        MetricRow {
            visible: root.primaryGpu && root.primaryGpu.memory_clock_mhz !== null
            Layout.fillWidth: true
            accentColor: theme.memoryColor
            label: "Mem clock"
            value: root.primaryGpu ? (root.primaryGpu.memory_clock_mhz || 0) + " MHz" : "-"
        }

        MetricRow {
            visible: root.primaryGpu && root.primaryGpu.fan_rpm !== null
            Layout.fillWidth: true
            accentColor: theme.systemColor
            label: "Fan"
            value: root.primaryGpu ? (root.primaryGpu.fan_rpm || 0) + " RPM" : "-"
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: "transparent"
            label: "Driver"
            value: root.primaryGpu ? root.vendorLabel(root.primaryGpu.vendor) : "-"
        }
    }

    // ── GPUs secundárias (compacto) ──────────────────────────────────────────
    Repeater {
        model: root.extraGpus

        delegate: MetricCard {
            Layout.fillWidth: true
            title: modelData.name || "GPU"
            subtitle: root.vendorLabel(modelData.vendor)

            MetricRow {
                Layout.fillWidth: true
                accentColor: theme.gpuColor
                label: "Uso"
                value: root.fmtPercent(modelData.usage_percent)
            }

            MetricRow {
                visible: modelData.temperature_celsius !== null
                Layout.fillWidth: true
                accentColor: root.temperatureAccentColor(modelData.temperature_celsius)
                label: "Temperatura"
                value: modelData.temperature_celsius !== null
                    ? Number(modelData.temperature_celsius).toFixed(1) + "°C" : "-"
            }

            MetricRow {
                visible: modelData.vram_total_gb !== null
                Layout.fillWidth: true
                accentColor: theme.memoryColor
                label: "VRAM"
                value: modelData.vram_total_gb !== null
                    ? theme.fmtOne(modelData.vram_used_gb) + " / " + theme.fmtOne(modelData.vram_total_gb) + " GB"
                    : "-"
            }
        }
    }

    Theme { id: theme }
}
