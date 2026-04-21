import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import "../components"
import ".."

ColumnLayout {
    id: root

    property var metrics: ({})

    // ── helpers ──────────────────────────────────────────────────────────────

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
                value: theme.fmtUptime(metrics ? metrics.uptime : 0)
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
        title: "Processos"
        subtitle: metrics && metrics.top_processes
            ? metrics.top_processes.length + " com maior uso de CPU"
            : "Coletando..."

        // Cabeçalho da tabela
        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingS

            PlasmaComponents3.Label {
                text: "Processo"
                font.bold: true
                font.pixelSize: 10
                color: theme.mutedTextColor
                Layout.fillWidth: true
            }
            PlasmaComponents3.Label {
                text: "CPU"
                font.bold: true
                font.pixelSize: 10
                color: theme.mutedTextColor
                Layout.preferredWidth: 44
                horizontalAlignment: Text.AlignRight
            }
            PlasmaComponents3.Label {
                text: "RAM"
                font.bold: true
                font.pixelSize: 10
                color: theme.mutedTextColor
                Layout.preferredWidth: 64
                horizontalAlignment: Text.AlignRight
            }
        }

        Repeater {
            model: metrics && metrics.top_processes ? metrics.top_processes : []

            delegate: RowLayout {
                Layout.fillWidth: true
                spacing: theme.spacingS

                Rectangle {
                    width: 6; height: 6; radius: 3
                    Layout.alignment: Qt.AlignVCenter
                    color: modelData.cpu_percent > 50 ? theme.dangerColor
                         : modelData.cpu_percent > 20 ? theme.warningColor
                         : theme.cpuColor
                }

                PlasmaComponents3.Label {
                    text: modelData.name
                    font.pixelSize: 11
                    color: theme.subduedTextColor
                    Layout.fillWidth: true
                    elide: Text.ElideRight
                }

                PlasmaComponents3.Label {
                    text: Number(modelData.cpu_percent).toFixed(1) + "%"
                    font.pixelSize: 11
                    font.bold: true
                    color: modelData.cpu_percent > 50 ? theme.dangerColor
                         : modelData.cpu_percent > 20 ? theme.warningColor
                         : theme.subduedTextColor
                    Layout.preferredWidth: 44
                    horizontalAlignment: Text.AlignRight
                }

                PlasmaComponents3.Label {
                    text: theme.fmtOne(modelData.memory_mb) + " MB"
                    font.pixelSize: 11
                    color: theme.mutedTextColor
                    Layout.preferredWidth: 64
                    horizontalAlignment: Text.AlignRight
                }
            }
        }
    }

    Theme { id: theme }
}
