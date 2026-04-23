import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.plasma.components 3.0 as PlasmaComponents3
import "../components"
import ".."

ColumnLayout {
    id: root

    property var systemInfo: ({})
    property var topProcesses: []
    property int uptime: 0
    property var loadAverage: [0, 0, 0]
    property var loadHistory: ({})
    property var processCountHistory: ({})
    property int historyDurationMs: 5 * 60 * 1000

    function historyWindowLabel() {
        return "Últimos " + Math.max(1, Math.round(historyDurationMs / 60000)) + " min";
    }

    function seriesLength(series) {
        return series && series.count !== undefined ? series.count : 0;
    }

    function seriesValue(series, index) {
        if (!series || !series.buffer || index < 0 || index >= root.seriesLength(series))
            return 0;
        var actualIndex = (series.start + index) % series.buffer.length;
        return Number(series.buffer[actualIndex] || 0);
    }

    function seriesMaximum(series, fallbackValue) {
        var maximum = fallbackValue || 1;
        for (var i = 0; i < root.seriesLength(series); i += 1)
            maximum = Math.max(maximum, root.seriesValue(series, i));
        return maximum;
    }

    Layout.fillWidth: true
    spacing: theme.spacingM

    MetricCard {
        Layout.fillWidth: true
        hero: true
        title: root.systemInfo.hostname || "Sistema"
        subtitle: (root.systemInfo.os_name || "Linux") + " " + (root.systemInfo.os_version || "")

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.spacingM

            HeroMetric {
                Layout.fillWidth: true
                label: "Uptime"
                value: theme.fmtUptime(root.uptime)
                accentColor: theme.systemColor
                footnote: "desde o boot"
            }

            HeroMetric {
                Layout.fillWidth: true
                label: "Processos"
                value: root.systemInfo.process_count ? String(root.systemInfo.process_count) : "-"
                accentColor: theme.cpuColor
                footnote: "em execução"
            }

            HeroMetric {
                Layout.fillWidth: true
                label: "Arquitetura"
                value: root.systemInfo.architecture || "-"
                accentColor: theme.systemColor
                footnote: root.systemInfo.kernel_version
                    ? root.systemInfo.kernel_version.split("-")[0]
                    : "kernel"
            }
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Sistema operacional"
        subtitle: root.systemInfo.os_name || "Linux"

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.systemColor
            label: "Distribuição"
            value: (root.systemInfo.os_name || "-") + " " + (root.systemInfo.os_version || "")
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.systemColor
            label: "Kernel"
            value: root.systemInfo.kernel_version || "-"
        }

        MetricRow {
            Layout.fillWidth: true
            accentColor: theme.systemColor
            label: "Hostname"
            value: root.systemInfo.hostname || "-"
        }
    }

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
                value: root.loadAverage ? Number(root.loadAverage[0]).toFixed(2) : "0.00"
                accentColor: theme.warningColor
            }

            HeroMetric {
                Layout.fillWidth: true
                label: "5 minutos"
                value: root.loadAverage ? Number(root.loadAverage[1]).toFixed(2) : "0.00"
                accentColor: theme.warningColor
            }

            HeroMetric {
                Layout.fillWidth: true
                label: "15 minutos"
                value: root.loadAverage ? Number(root.loadAverage[2]).toFixed(2) : "0.00"
                accentColor: theme.warningColor
            }
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Load history"
        subtitle: root.historyWindowLabel()

        HistoryChart {
            Layout.fillWidth: true
            series: root.loadHistory
            strokeColor: theme.warningColor
            maximumValue: Math.max(root.seriesMaximum(root.loadHistory, 1), 1)
            maxLabel: Number(root.seriesMaximum(root.loadHistory, 1)).toFixed(2)
            minLabel: "0.00"
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Process count history"
        subtitle: root.historyWindowLabel()

        HistoryChart {
            Layout.fillWidth: true
            series: root.processCountHistory
            strokeColor: theme.cpuColor
            maximumValue: Math.max(root.seriesMaximum(root.processCountHistory, 1), 1)
            maxLabel: Math.round(root.seriesMaximum(root.processCountHistory, 1)).toString()
            minLabel: "0"
        }
    }

    MetricCard {
        Layout.fillWidth: true
        title: "Processos"
        subtitle: root.topProcesses ? root.topProcesses.length + " com maior uso de CPU" : "Coletando..."

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
            model: root.topProcesses || []

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
