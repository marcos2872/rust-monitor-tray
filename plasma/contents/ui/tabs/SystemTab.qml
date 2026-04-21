import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import "../components"
import ".."

ColumnLayout {
    id: root

    property var metrics: ({})

    // ── helpers ──────────────────────────────────────────────────────────────

    function fmtUptime(seconds) {
        if (seconds === undefined || seconds === null || isNaN(seconds)) return "0m";
        var total   = Number(seconds);
        var days    = Math.floor(total / 86400);
        var hours   = Math.floor((total % 86400) / 3600);
        var minutes = Math.floor((total % 3600) / 60);
        if (days > 0)  return days + "d " + hours + "h " + minutes + "m";
        if (hours > 0) return hours + "h " + minutes + "m";
        return minutes + "m";
    }

    function fmtOne(value) {
        if (value === undefined || value === null || isNaN(value)) return "0.0";
        return Number(value).toFixed(1);
    }

    function fmtBytes(value) {
        if (value === undefined || value === null || isNaN(value)) return "0 B";
        var units = ["B", "KB", "MB", "GB", "TB"];
        var size  = Number(value);
        var index = 0;
        while (size >= 1024 && index < units.length - 1) { size /= 1024; index += 1; }
        return size.toFixed(index === 0 ? 0 : 1) + " " + units[index];
    }

    function sysInfo() {
        return metrics && metrics.system_info ? metrics.system_info : {};
    }

    Layout.fillWidth: true
    spacing: theme.spacingM

    // ── Hero: identificação do host ──────────────────────────────────────────
    MetricCard {
        Layout.fillWidth: true
        hero: true
        title: sysInfo().hostname || "Sistema"
        subtitle: (sysInfo().os_name || "Linux") + " " + (sysInfo().os_version || "")

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingM

            HeroMetric {
                Layout.fillWidth: true
                label: "Uptime"
                value: root.fmtUptime(metrics ? metrics.uptime : 0)
                accentColor: theme.systemColor
                footnote: "desde o boot"
            }

            HeroMetric {
                Layout.fillWidth: true
                label: "Processos"
                value: sysInfo().process_count ? String(sysInfo().process_count) : "-"
                accentColor: theme.cpuColor
                footnote: "em execução"
            }

            HeroMetric {
                Layout.fillWidth: true
                label: "Arquitetura"
                value: sysInfo().architecture || "-"
                accentColor: theme.systemColor
                footnote: sysInfo().kernel_version
                    ? sysInfo().kernel_version.split("-")[0]
                    : "kernel"
            }
        }
    }

    // ── Kernel + OS ──────────────────────────────────────────────────────────
    MetricCard {
        Layout.fillWidth: true
        title: "Sistema operacional"
        subtitle: sysInfo().os_name || "Linux"

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.systemColor
            label: "Distribuição"
            value: (sysInfo().os_name || "-") + " " + (sysInfo().os_version || "")
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.systemColor
            label: "Kernel"
            value: sysInfo().kernel_version || "-"
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.systemColor
            label: "Hostname"
            value: sysInfo().hostname || "-"
        }
    }

    // ── Load average ─────────────────────────────────────────────────────────
    MetricCard {
        Layout.fillWidth: true
        title: "Load average"
        subtitle: "Média de carga do sistema"

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingM

            HeroMetric {
                Layout.fillWidth: true
                label: "1 minuto"
                value: metrics && metrics.load_average
                    ? Number(metrics.load_average[0]).toFixed(2) : "0.00"
                accentColor: theme.warningColor
            }

            HeroMetric {
                Layout.fillWidth: true
                label: "5 minutos"
                value: metrics && metrics.load_average
                    ? Number(metrics.load_average[1]).toFixed(2) : "0.00"
                accentColor: theme.warningColor
            }

            HeroMetric {
                Layout.fillWidth: true
                label: "15 minutos"
                value: metrics && metrics.load_average
                    ? Number(metrics.load_average[2]).toFixed(2) : "0.00"
                accentColor: theme.warningColor
            }
        }
    }

    // ── Resumo de recursos ───────────────────────────────────────────────────
    MetricCard {
        Layout.fillWidth: true
        title: "Recursos"
        subtitle: "Visão geral do hardware"

        // CPU
        SectionHeader { title: "CPU" }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.cpuColor
            label: metrics && metrics.cpu ? metrics.cpu.name : "-"
            value: metrics && metrics.cpu
                ? metrics.cpu.core_count + " núcleos · " + metrics.cpu.frequency + " MHz"
                : "-"
        }

        MetricBar {
            Layout.fillWidth: true
            label: "Uso"
            value: metrics && metrics.cpu ? metrics.cpu.usage_percent : 0
            barColor: theme.cpuColor
        }

        // RAM
        SectionHeader { title: "Memória" }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.memoryColor
            label: "RAM"
            value: metrics && metrics.memory
                ? root.fmtOne(metrics.memory.used_memory) + " / " + root.fmtOne(metrics.memory.total_memory) + " GB"
                : "-"
        }

        MetricBar {
            Layout.fillWidth: true
            label: "Uso"
            value: metrics && metrics.memory ? metrics.memory.usage_percent : 0
            barColor: theme.memoryColor
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: "#a78bfa"
            label: "Swap"
            value: metrics && metrics.memory
                ? root.fmtOne(metrics.memory.used_swap) + " / " + root.fmtOne(metrics.memory.total_swap) + " GB"
                : "-"
        }

        // Disco
        SectionHeader { title: "Disco" }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.diskColor
            label: "Armazenamento"
            value: metrics && metrics.disk
                ? root.fmtOne(metrics.disk.used_space) + " / " + root.fmtOne(metrics.disk.total_space) + " GB"
                : "-"
        }

        MetricBar {
            Layout.fillWidth: true
            label: "Uso"
            value: metrics && metrics.disk
                ? (metrics.disk.total_space > 0
                    ? (metrics.disk.used_space / metrics.disk.total_space * 100)
                    : 0)
                : 0
            barColor: theme.diskColor
        }

        // Rede
        SectionHeader { title: "Rede" }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.cpuColor
            label: "↓ Recebido total"
            value: metrics && metrics.network
                ? root.fmtBytes(metrics.network.total_bytes_received)
                : "-"
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.dangerColor
            label: "↑ Enviado total"
            value: metrics && metrics.network
                ? root.fmtBytes(metrics.network.total_bytes_transmitted)
                : "-"
        }

        // Temperatura
        SectionHeader { title: "Temperatura" }

        MetricRow {
            Layout.fillWidth: true
            accentColor: {
                var t = metrics && metrics.sensors
                    ? (metrics.sensors.hottest_temperature_celsius || 0) : 0;
                if (t >= 85) return theme.dangerColor;
                if (t >= 70) return theme.warningColor;
                return theme.successColor;
            }
            label: metrics && metrics.sensors && metrics.sensors.hottest_label
                ? metrics.sensors.hottest_label
                : "Sensor mais quente"
            value: metrics && metrics.sensors
                && metrics.sensors.hottest_temperature_celsius !== null
                && metrics.sensors.hottest_temperature_celsius !== undefined
                ? Number(metrics.sensors.hottest_temperature_celsius).toFixed(1) + "°C"
                : "-"
        }
    }

    Theme { id: theme }
}
