import QtQuick 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.20 as Kirigami
import org.kde.plasma.components 3.0 as PlasmaComponents3
import "components"

PlasmaComponents3.ScrollView {
    id: root

    property var metrics: ({})
    property string errorMessage: ""

    function asArray(mapObject) {
        var rows = [];
        if (!mapObject) {
            return rows;
        }
        for (var key in mapObject) {
            rows.push({ name: key, data: mapObject[key] });
        }
        rows.sort(function(a, b) { return a.name.localeCompare(b.name); });
        return rows;
    }

    function fmtPercent(value) {
        if (value === undefined || value === null || isNaN(value)) {
            return "0%";
        }
        return Math.round(value) + "%";
    }

    function fmtOne(value) {
        if (value === undefined || value === null || isNaN(value)) {
            return "0.0";
        }
        return Number(value).toFixed(1);
    }

    function usageColor(value, defaultColor) {
        if (value >= 80) {
            return theme.dangerColor;
        }
        return defaultColor;
    }

    function fmtBytes(value) {
        if (value === undefined || value === null || isNaN(value)) {
            return "0 B";
        }
        var units = ["B", "KB", "MB", "GB", "TB"];
        var size = Number(value);
        var index = 0;
        while (size >= 1024 && index < units.length - 1) {
            size /= 1024;
            index += 1;
        }
        return size.toFixed(index === 0 ? 0 : 1) + " " + units[index];
    }

    function fmtUptime(seconds) {
        if (seconds === undefined || seconds === null || isNaN(seconds)) {
            return "0m";
        }
        var total = Number(seconds);
        var days = Math.floor(total / 86400);
        var hours = Math.floor((total % 86400) / 3600);
        var minutes = Math.floor((total % 3600) / 60);
        if (days > 0) {
            return days + "d " + hours + "h " + minutes + "m";
        }
        if (hours > 0) {
            return hours + "h " + minutes + "m";
        }
        return minutes + "m";
    }

    clip: true
    contentWidth: availableWidth

    ColumnLayout {
        width: root.availableWidth
        spacing: theme.spacingM

        SectionHeader {
            Layout.fillWidth: true
            title: "Monitor do Sistema"
            subtitle: root.errorMessage.length > 0
                ? root.errorMessage
                : "Atualização contínua via backend monitor-tray"
        }

        MetricCard {
            visible: root.errorMessage.length === 0
            title: "CPU"
            subtitle: metrics.cpu ? metrics.cpu.name : "Sem dados"

            MetricBar {
                label: "Uso total"
                value: metrics.cpu ? metrics.cpu.usage_percent : 0
                barColor: usageColor(metrics.cpu ? metrics.cpu.usage_percent : 0, theme.cpuColor)
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: theme.spacingM

                MetricRow {
                    Layout.fillWidth: true
                    label: "Freq"
                    value: metrics.cpu ? metrics.cpu.frequency + " MHz" : "-"
                }

                MetricRow {
                    Layout.fillWidth: true
                    label: "Núcleos"
                    value: metrics.cpu ? metrics.cpu.core_count : "-"
                }
            }

            SectionHeader {
                title: "Uso por núcleo"
            }

            GridLayout {
                Layout.fillWidth: true
                columns: 2
                columnSpacing: theme.spacingS
                rowSpacing: theme.spacingXS

                Repeater {
                    model: metrics.cpu && metrics.cpu.per_core_usage ? metrics.cpu.per_core_usage : []

                    delegate: MetricRow {
                        Layout.fillWidth: true
                        label: "Core " + String(index + 1).padStart(2, '0')
                        value: fmtPercent(modelData)
                    }
                }
            }
        }

        MetricCard {
            visible: root.errorMessage.length === 0
            title: "Memória"
            subtitle: metrics.memory ? (fmtOne(metrics.memory.used_memory) + " / " + fmtOne(metrics.memory.total_memory) + " GB") : "Sem dados"

            MetricBar {
                label: "RAM"
                value: metrics.memory ? metrics.memory.usage_percent : 0
                barColor: usageColor(metrics.memory ? metrics.memory.usage_percent : 0, theme.memoryColor)
            }

            MetricRow {
                label: "Usada"
                value: metrics.memory ? fmtOne(metrics.memory.used_memory) + " GB" : "-"
            }

            MetricRow {
                label: "Livre"
                value: metrics.memory ? fmtOne(metrics.memory.available_memory) + " GB" : "-"
            }

            MetricBar {
                visible: metrics.memory && metrics.memory.total_swap > 0
                label: "Swap"
                value: metrics.memory && metrics.memory.total_swap > 0 ? (metrics.memory.used_swap / metrics.memory.total_swap) * 100.0 : 0
                barColor: "#a78bfa"
            }
        }

        MetricCard {
            visible: root.errorMessage.length === 0
            title: "Disco"
            subtitle: metrics.disk ? (fmtOne(metrics.disk.used_space) + " / " + fmtOne(metrics.disk.total_space) + " GB") : "Sem dados"

            MetricBar {
                label: "Uso agregado"
                value: metrics.disk && metrics.disk.total_space > 0 ? (metrics.disk.used_space / metrics.disk.total_space) * 100.0 : 0
                barColor: usageColor(metrics.disk && metrics.disk.total_space > 0 ? (metrics.disk.used_space / metrics.disk.total_space) * 100.0 : 0, theme.diskColor)
            }

            SectionHeader {
                title: "Partições"
            }

            Repeater {
                model: metrics.disk && metrics.disk.disks ? metrics.disk.disks : []

                delegate: MetricRow {
                    Layout.fillWidth: true
                    label: modelData.mount_point
                    value: fmtPercent(modelData.usage_percent)
                }
            }
        }

        MetricCard {
            visible: root.errorMessage.length === 0
            title: "Rede"
            subtitle: metrics.network ? (asArray(metrics.network.interfaces).length + " interfaces") : "Sem dados"

            RowLayout {
                Layout.fillWidth: true
                spacing: theme.spacingM

                MetricRow {
                    Layout.fillWidth: true
                    label: "RX"
                    value: metrics.network ? fmtBytes(metrics.network.total_bytes_received) : "-"
                }

                MetricRow {
                    Layout.fillWidth: true
                    label: "TX"
                    value: metrics.network ? fmtBytes(metrics.network.total_bytes_transmitted) : "-"
                }
            }

            SectionHeader {
                title: "Interfaces"
            }

            Repeater {
                model: asArray(metrics.network ? metrics.network.interfaces : null)

                delegate: MetricRow {
                    Layout.fillWidth: true
                    label: modelData.name
                    value: "↓ " + fmtBytes(modelData.data.bytes_received) + " · ↑ " + fmtBytes(modelData.data.bytes_transmitted)
                }
            }
        }

        MetricCard {
            visible: root.errorMessage.length === 0
            title: "Sistema"
            subtitle: "Estado geral"

            MetricRow {
                label: "Uptime"
                value: fmtUptime(metrics.uptime)
            }

            MetricRow {
                label: "Load avg"
                value: metrics.load_average ? (Number(metrics.load_average[0]).toFixed(2) + " / " + Number(metrics.load_average[1]).toFixed(2) + " / " + Number(metrics.load_average[2]).toFixed(2)) : "-"
            }
        }

        MetricCard {
            visible: root.errorMessage.length > 0
            title: "Backend indisponível"
            subtitle: "O Plasmoid não conseguiu obter métricas do executável monitor-tray"

            PlasmaComponents3.Label {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                text: root.errorMessage
            }
        }
    }

    Theme {
        id: theme
    }
}
